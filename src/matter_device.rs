use core::pin::pin;

use std::sync::Arc;

use esp_idf_matter::init_async_io;
use esp_idf_matter::matter::crypto::default_crypto;
use esp_idf_matter::matter::dm::devices::test::{
    DAC_PRIVKEY, TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET,
};
use esp_idf_matter::matter::dm::{EmptyHandler, Node};
use esp_idf_matter::matter::utils::init::InitMaybeUninit;
use esp_idf_matter::persist::EspKvBlobStore;
use esp_idf_matter::wireless::{EspMatterWifi, EspWifiMatterStack};

use esp_idf_svc::bt::reduce_bt_memory;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::io::vfs::MountedEventfs;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::timer::EspTaskTimerService;

use log::{error, info};

use static_cell::StaticCell;

const BUMP_SIZE: usize = 20000;

static MATTER_STACK: StaticCell<EspWifiMatterStack<BUMP_SIZE, ()>> = StaticCell::new();

/// Endpoint layout (established here, stable for future slices):
/// - Endpoint 0: root node — standard Matter commissioning clusters
/// - Endpoint 1: reserved for the humidity sensor (slice 03: RelativeHumidityMeasurement + On/Off)
/// - Endpoint 2+: reserved for a future watering actuator
const NODE: Node = Node {
    endpoints: &[EspWifiMatterStack::<0, ()>::root_endpoint()],
};

pub fn run(modem: Modem) -> Result<(), anyhow::Error> {
    let result = block_on(matter(modem));

    if let Err(e) = &result {
        error!("Matter aborted execution with error: {e:?}");
    }

    result
}

async fn matter(mut modem: Modem<'_>) -> Result<(), anyhow::Error> {
    let stack = MATTER_STACK.uninit().init_with(EspWifiMatterStack::init(
        &TEST_DEV_DET,
        TEST_DEV_COMM,
        &TEST_DEV_ATT,
    ));

    let sysloop = EspSystemEventLoop::take()?;
    let timers = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let _mounted_event_fs = Arc::new(MountedEventfs::mount(3)?);
    init_async_io(_mounted_event_fs.clone())?;

    reduce_bt_memory(unsafe { modem.reborrow() })?;

    let crypto = default_crypto(rand::thread_rng(), DAC_PRIVKEY);

    let handler = EmptyHandler;

    let mut store = EspKvBlobStore::new_default(nvs.clone())?;
    stack.startup(&crypto, &mut store).await?;

    if stack.is_commissioned() {
        info!("Device is commissioned; operational traffic will run over WiFi");
    } else {
        info!("Device is not commissioned; scan the QR code to commission over BLE");
    }

    let kv = stack.kv(&mut store);

    let matter = pin!(stack.run_coex(
        EspMatterWifi::new_with_builtin_mdns(modem, sysloop, timers, nvs, stack),
        &crypto,
        (NODE, handler),
        kv,
        (),
    ));

    matter.await?;
    Ok(())
}
