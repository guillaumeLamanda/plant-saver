//! H2 Thread Matter wireless shim.
//!
//! The `esp_idf_matter::wireless` module is gated `not(esp32h2)` in the pinned
//! crate revision, so EspThreadMatterStack / EspMatterThread aren't available.
//! This module provides the minimum types for a Thread+BLE Matter stack on H2,
//! replicating `wireless/thread.rs` and the `EspGatt` embedding.

use alloc::sync::Arc;

use esp_idf_matter::ble::{EspBtpGattContext, EspBtpGattPeripheral};
use esp_idf_matter::error::to_net_error;
use esp_idf_matter::netif::{EspMatterNetStack, EspMatterNetif};
use esp_idf_matter::thread::{EspMatterThreadCtl, EspMatterThreadSrp};

use esp_idf_svc::bt::{self, BtDriver};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::io::vfs::MountedEventfs;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::thread::{EspThread, Node};

use log::info;

use rs_matter_stack::matter::dm::clusters::gen_diag::InterfaceTypeEnum;
use rs_matter_stack::matter::dm::networks::wireless::Thread;
use rs_matter_stack::matter::error::Error;
use rs_matter_stack::matter::utils::init::{init, Init};
use rs_matter_stack::network::{Embedding, Network};
use rs_matter_stack::wireless::{
    Gatt, GattTask, ThreadCoex, ThreadCoexTask, ThreadTask, WirelessBle,
};
use rs_matter_stack::MatterStack;

extern crate alloc;

const GATTS_APP_ID: u16 = 0;

pub type EspThreadMatterStack<'a, const B: usize, E> =
    MatterStack<'a, B, WirelessBle<Thread, EspGattH2<E>>>;

pub struct EspGattH2<E = ()> {
    btp_gatt_context: EspBtpGattContext,
    embedding: E,
}

impl<E> EspGattH2<E>
where
    E: Embedding,
{
    const fn new() -> Self {
        Self {
            btp_gatt_context: EspBtpGattContext::new(),
            embedding: E::INIT,
        }
    }

    fn init() -> impl Init<Self> {
        init!(Self {
            btp_gatt_context <- EspBtpGattContext::init(),
            embedding <- E::init(),
        })
    }

    pub fn context(&self) -> &EspBtpGattContext {
        &self.btp_gatt_context
    }
}

impl<E> Embedding for EspGattH2<E>
where
    E: Embedding,
{
    const INIT: Self = Self::new();

    fn init() -> impl Init<Self> {
        EspGattH2::init()
    }
}

pub struct EspMatterThread<'a, 'd> {
    modem: Modem<'d>,
    sysloop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
    mounted_event_fs: Arc<MountedEventfs>,
    ble_context: &'a EspBtpGattContext,
}

impl<'a, 'd> EspMatterThread<'a, 'd> {
    pub fn new<const B: usize, E>(
        modem: Modem<'d>,
        sysloop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
        mounted_event_fs: Arc<MountedEventfs>,
        stack: &'a EspThreadMatterStack<B, E>,
    ) -> Self
    where
        E: Embedding + 'static,
    {
        Self::wrap(
            modem,
            sysloop,
            nvs,
            mounted_event_fs,
            stack.network().embedding().context(),
        )
    }

    pub fn wrap(
        modem: Modem<'d>,
        sysloop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
        mounted_event_fs: Arc<MountedEventfs>,
        ble_context: &'a EspBtpGattContext,
    ) -> Self {
        Self {
            modem,
            sysloop,
            nvs,
            mounted_event_fs,
            ble_context,
        }
    }
}

impl Gatt for EspMatterThread<'_, '_> {
    async fn run<A>(&mut self, mut task: A) -> Result<(), Error>
    where
        A: GattTask,
    {
        let bt = BtDriver::new(unsafe { self.modem.reborrow() }, Some(self.nvs.clone())).unwrap();
        let peripheral =
            EspBtpGattPeripheral::<bt::Ble>::new(GATTS_APP_ID, bt, self.ble_context).unwrap();

        task.run(peripheral).await
    }
}

impl rs_matter_stack::wireless::Thread for EspMatterThread<'_, '_> {
    type NetCtl<'a>
        = &'a EspMatterThreadCtl<'a, 'a, Node>
    where
        Self: 'a;

    async fn run<A>(&mut self, mut task: A) -> Result<(), Error>
    where
        A: ThreadTask,
    {
        let mut thread = EspThread::new(
            unsafe { self.modem.reborrow() },
            self.sysloop.clone(),
            self.nvs.clone(),
            self.mounted_event_fs.clone(),
        )
        .map_err(to_net_error)?;

        thread.enable_ipv6(true).map_err(to_net_error)?;
        thread.enable_thread(true).map_err(to_net_error)?;

        info!("Thread stack created, about to start it");

        thread.start().map_err(to_net_error)?;

        info!("Thread stack started");

        let net_ctl = EspMatterThreadCtl::new(&thread, self.sysloop.clone());
        let mut mdns = EspMatterThreadSrp::new(&thread);

        task.run(
            EspMatterNetStack::new(),
            EspMatterNetif::new(&net_ctl, InterfaceTypeEnum::Thread, self.sysloop.clone()),
            &net_ctl,
            &mut mdns,
        )
        .await
    }
}

impl ThreadCoex for EspMatterThread<'_, '_> {
    async fn run<A>(&mut self, mut task: A) -> Result<(), Error>
    where
        A: ThreadCoexTask,
    {
        let modem = unsafe { self.modem.reborrow() };

        let (thread_p, bt_p) = modem.split();

        let mut thread = EspThread::new(
            thread_p,
            self.sysloop.clone(),
            self.nvs.clone(),
            self.mounted_event_fs.clone(),
        )
        .map_err(to_net_error)?;

        thread.enable_ipv6(true).map_err(to_net_error)?;
        thread.enable_thread(true).map_err(to_net_error)?;

        info!("Thread stack created, about to start it");

        thread.start().map_err(to_net_error)?;

        info!("Thread stack started");

        let net_ctl = EspMatterThreadCtl::new(&thread, self.sysloop.clone());
        let mut mdns = EspMatterThreadSrp::new(&thread);
        let bt = BtDriver::new(bt_p, Some(self.nvs.clone())).unwrap();
        let mut peripheral =
            EspBtpGattPeripheral::<bt::Ble>::new(GATTS_APP_ID, bt, self.ble_context).unwrap();

        task.run(
            EspMatterNetStack::new(),
            EspMatterNetif::new(&net_ctl, InterfaceTypeEnum::Thread, self.sysloop.clone()),
            &net_ctl,
            &mut mdns,
            &mut peripheral,
        )
        .await
    }
}
