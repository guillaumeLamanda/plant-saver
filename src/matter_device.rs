use core::pin::pin;

use std::sync::Arc;

use esp_idf_matter::init_async_io;
use esp_idf_matter::matter::crypto::{default_crypto, Crypto};
use esp_idf_matter::matter::dm::clusters::desc::{self, ClusterHandler as _, DescHandler};
use esp_idf_matter::matter::dm::devices::test::{
    DAC_PRIVKEY, TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET,
};
use esp_idf_matter::matter::dm::{
    Async, Dataver, DeviceType, EmptyHandler, Endpoint, EpClMatcher, Node,
};
use esp_idf_matter::matter::utils::init::InitMaybeUninit;
use esp_idf_matter::matter::{clusters, devices};
use esp_idf_matter::persist::EspKvBlobStore;

use esp_idf_svc::bt::reduce_bt_memory;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::io::vfs::MountedEventfs;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use log::info;

use static_cell::StaticCell;

use crate::log_heap;
use crate::matter_clusters::humidity::{self, RelativeHumidityHandler, HUMIDITY_ENDPOINT_ID};

const BUMP_SIZE: usize = 12000;

use crate::h2_wireless::{EspMatterThread, EspThreadMatterStack};
static MATTER_STACK: StaticCell<EspThreadMatterStack<BUMP_SIZE, ()>> = StaticCell::new();

const DEV_TYPE_HUMIDITY_SENSOR: DeviceType = DeviceType {
    dtype: 0x0307,
    drev: 1,
};

#[cfg(esp32h2)]
const NODE: Node = Node {
    endpoints: &[
        EspThreadMatterStack::<0, ()>::root_endpoint(),
        Endpoint::new(
            HUMIDITY_ENDPOINT_ID,
            devices!(DEV_TYPE_HUMIDITY_SENSOR),
            clusters!(DescHandler::CLUSTER, humidity::HUMIDITY_CLUSTER),
        ),
    ],
};

pub async fn matter(
    mut modem: Modem<'_>,
    mut get_value: impl FnMut() -> u8,
) -> Result<(), anyhow::Error> {
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    log_heap("pre-reduce-bt-memory");
    reduce_bt_memory(unsafe { modem.reborrow() })?;
    log_heap("post-reduce-bt-memory");

    let crypto = default_crypto(rand::thread_rng(), DAC_PRIVKEY);
    let mut weak_rand = crypto.weak_rand()?;

    let humidity_handler =
        RelativeHumidityHandler::new(Dataver::new_rand(&mut weak_rand), get_value());

    let handler = EmptyHandler
        .chain(
            EpClMatcher::new(
                Some(HUMIDITY_ENDPOINT_ID),
                Some(humidity::HUMIDITY_CLUSTER.id),
            ),
            Async(humidity_handler.adapt()),
        )
        .chain(
            EpClMatcher::new(Some(HUMIDITY_ENDPOINT_ID), Some(DescHandler::CLUSTER.id)),
            Async(desc::DescHandler::new(Dataver::new_rand(&mut weak_rand)).adapt()),
        );

    let mut store = EspKvBlobStore::new_default(nvs.clone())?;

    log_heap("pre-matter-h2-init");

    let stack = MATTER_STACK.uninit().init_with(EspThreadMatterStack::init(
        &TEST_DEV_DET,
        TEST_DEV_COMM,
        &TEST_DEV_ATT,
    ));
    stack.startup(&crypto, &mut store).await?;

    if stack.is_commissioned() {
        info!("Device is commissioned; operational traffic will run over Thread");
    } else {
        info!("Device is not commissioned; scan the QR code to commission over BLE");
    }

    let kv = stack.kv(&mut store);
    let mounted_event_fs = Arc::new(MountedEventfs::mount(6)?);
    init_async_io(mounted_event_fs.clone())?;

    let matter = pin!(stack.run_coex(
        EspMatterThread::new(modem, sysloop, nvs, mounted_event_fs, stack),
        &crypto,
        (NODE, handler),
        kv,
        (),
    ));

    matter.await?;

    Ok(())
}
