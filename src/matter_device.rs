use core::pin::pin;

use std::sync::Arc;

use esp_idf_matter::init_async_io;
use esp_idf_matter::matter::crypto::{Crypto, default_crypto};
use esp_idf_matter::matter::dm::clusters::desc::{self, ClusterHandler as _, DescHandler};
use esp_idf_matter::matter::dm::clusters::identify;
use esp_idf_matter::matter::dm::devices::test::{
    DAC_PRIVKEY, TEST_DEV_ATT, TEST_DEV_COMM, TEST_DEV_DET,
};
use esp_idf_matter::matter::dm::endpoints::ROOT_ENDPOINT_ID;
use esp_idf_matter::matter::dm::{Async, Dataver, DeviceType, EmptyHandler, Endpoint, Node};
use esp_idf_matter::matter::utils::init::InitMaybeUninit;
use esp_idf_matter::matter::{clusters, devices};
use esp_idf_matter::persist::EspKvBlobStore;

use esp_idf_svc::bt::reduce_bt_memory;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::io::vfs::MountedEventfs;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};

use log::info;

use rand::Rng as _;

use static_cell::StaticCell;

use crate::log_heap;
use crate::matter_clusters::humidity::{
    self, HUMIDITY_ENDPOINT_ID, RelativeHumidityHandler, SharedSoilMoistureLevel,
};

const BUMP_SIZE: usize = 17000;

use esp_idf_matter::wireless::{EspMatterThread, EspThreadMatterStack};
static MATTER_STACK: StaticCell<EspThreadMatterStack<BUMP_SIZE, ()>> = StaticCell::new();

// Revision 3 is the Humidity Sensor device type revision in Matter 1.6.
const DEV_TYPE_HUMIDITY_SENSOR: DeviceType = DeviceType {
    dtype: 0x0307,
    drev: 3,
};

const NODE: Node = Node {
    endpoints: &[
        EspThreadMatterStack::<0, ()>::root_endpoint(),
        Endpoint::new(
            HUMIDITY_ENDPOINT_ID,
            devices!(DEV_TYPE_HUMIDITY_SENSOR),
            clusters!(
                DescHandler::CLUSTER,
                identify::CLUSTER,
                humidity::HUMIDITY_CLUSTER
            ),
        ),
    ],
};

pub async fn matter(
    mut modem: Modem<'_>,
    soil_moisture_level: Arc<SharedSoilMoistureLevel>,
) -> Result<(), anyhow::Error> {
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mounted_event_fs = Arc::new(MountedEventfs::mount(6)?);
    init_async_io(mounted_event_fs.clone())?;

    log_heap("pre-reduce-bt-memory");
    reduce_bt_memory(unsafe { modem.reborrow() })?;
    log_heap("post-reduce-bt-memory");

    let crypto = default_crypto(rand::rng(), DAC_PRIVKEY);
    let mut weak_rand = crypto.weak_rand()?;

    let humidity_handler =
        RelativeHumidityHandler::new(Dataver::new_rand(&mut weak_rand), soil_moisture_level);

    let handler = EmptyHandler
        .chain(
            |e, _| e == ROOT_ENDPOINT_ID,
            Async(EspThreadMatterStack::<0, ()>::root_handler(
                &(),
                &mut weak_rand,
            )),
        )
        .chain(
            |e, c| e == HUMIDITY_ENDPOINT_ID && c == humidity::HUMIDITY_CLUSTER.id,
            Async(humidity_handler.adapt()),
        )
        .chain(
            |e, c| e == HUMIDITY_ENDPOINT_ID && c == DescHandler::CLUSTER.id,
            Async(desc::DescHandler::new(Dataver::new_rand(&mut weak_rand)).adapt()),
        )
        .chain(
            |e, c| e == HUMIDITY_ENDPOINT_ID && c == identify::CLUSTER.id,
            Async(identify::IdentifyHandler::new(Dataver::new_rand(&mut weak_rand)).adapt()),
        );

    let mut store = EspKvBlobStore::new_default(nvs.clone())?;

    log_heap("pre-matter-h2-init");

    let stack = MATTER_STACK.uninit().init_with(EspThreadMatterStack::init(
        &TEST_DEV_DET,
        TEST_DEV_COMM,
        &TEST_DEV_ATT,
    ));
    stack.startup(&crypto, &mut store).await?;

    if stack.matter().has_fabrics() {
        info!("Device is commissioned; operational traffic will run over Thread");
    } else {
        info!("Device is not commissioned; scan the QR code to commission over BLE");
    }

    let kv = stack.kv(&mut store);

    let matter = pin!(
        stack.run_coex(
            EspMatterThread::new(modem, sysloop, nvs.clone(), mounted_event_fs, stack)
                .with_srp_host_eui64(srp_host_eui64(nvs)?),
            &crypto,
            (NODE, handler),
            kv,
            (),
        )
    );

    matter.await?;

    Ok(())
}

const SRP_NVS_NAMESPACE: &str = "plant_saver";
const SRP_HOST_EUI64_KEY: &str = "srp_eui64";

/// EUI-64 the Thread SRP host name is derived from.
///
/// By default the host name comes from the factory MAC, so it never changes, while the
/// ECDSA key that owns it on the SRP server lives in NVS. A flash erase therefore
/// keeps the name but drops the key, and the border router rejects every
/// registration under that name (`Duplicated`) until the old key lease expires,
/// up to 14 days. The device then can't be found over operational mDNS, and
/// commissioning fails. Here a random EUI-64 is generated once and stored in NVS
/// next to the key, so an erase resets both.
fn srp_host_eui64(nvs: EspDefaultNvsPartition) -> Result<[u8; 8], anyhow::Error> {
    let nvs = EspNvs::new(nvs, SRP_NVS_NAMESPACE, true)?;

    let mut eui64 = [0; 8];
    if let Some(stored) = nvs.get_blob(SRP_HOST_EUI64_KEY, &mut eui64)?
        && stored.len() == eui64.len()
    {
        return Ok(eui64);
    }

    rand::rng().fill_bytes(&mut eui64);
    nvs.set_blob(SRP_HOST_EUI64_KEY, &eui64)?;
    info!("Generated a new SRP host EUI-64");

    Ok(eui64)
}
