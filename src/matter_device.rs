use core::pin::pin;

use std::sync::atomic::AtomicU8;
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

use crate::matter_clusters::humidity::{self, RelativeHumidityHandler, HUMIDITY_ENDPOINT_ID};

const BUMP_SIZE: usize = 20000;

static MATTER_STACK: StaticCell<EspWifiMatterStack<BUMP_SIZE, ()>> = StaticCell::new();

/// Humidity Sensor device type (CSA device library 0x0307, introduced in
/// Matter 1.3 → revision 1). Hosts the `RelativeHumidityMeasurement` cluster.
/// Used with test DACs for this phase.
const DEV_TYPE_HUMIDITY_SENSOR: DeviceType = DeviceType {
    dtype: 0x0307,
    drev: 1,
};

/// Endpoint layout (stable for future slices):
/// - Endpoint 0: root node — standard Matter commissioning clusters
/// - Endpoint 1: humidity sensor (`RelativeHumidityMeasurement`; slice 04 adds
///   a read-only On/Off "needs water" cluster here)
/// - Endpoint 2+: reserved for a future watering actuator
const NODE: Node = Node {
    endpoints: &[
        EspWifiMatterStack::<0, ()>::root_endpoint(),
        Endpoint::new(
            HUMIDITY_ENDPOINT_ID,
            devices!(DEV_TYPE_HUMIDITY_SENSOR),
            clusters!(DescHandler::CLUSTER, humidity::HUMIDITY_CLUSTER),
        ),
    ],
};

pub fn run(modem: Modem, soil_moisture_level: Arc<AtomicU8>) -> Result<(), anyhow::Error> {
    let result = block_on(matter(modem, soil_moisture_level));

    if let Err(e) = &result {
        error!("Matter aborted execution with error: {e:?}");
    }

    result
}

async fn matter(
    mut modem: Modem<'_>,
    soil_moisture_level: Arc<AtomicU8>,
) -> Result<(), anyhow::Error> {
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

    let mut weak_rand = crypto.weak_rand()?;

    // Pull-based humidity cluster handler: `measured_value` reads the latest
    // `SoilMoistureLevel` from the shared cell the main loop updates.
    let humidity_handler =
        RelativeHumidityHandler::new(Dataver::new_rand(&mut weak_rand), soil_moisture_level);

    // Chain endpoint 1 clusters with the (root) endpoint 0 system clusters.
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
