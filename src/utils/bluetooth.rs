use std::sync::{Arc};
use esp32_nimble::{enums::*, uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties, BLECharacteristic};
use esp32_nimble::utilities::mutex::Mutex;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};

pub struct Bluetooth{
    pub characteristic:Arc<Mutex<BLECharacteristic>>,
}
pub fn init() -> Result<Bluetooth, Box<dyn std::error::Error>> {

    let nvs = EspDefaultNvsPartition::take()?;
    let mut nvs_hander = EspNvs::new(nvs, "TEST", true)? ;


    let ble_device = BLEDevice::take();

    // Obtain handle for peripheral advertiser
    let ble_advertiser = ble_device.get_advertising();

    // Configure Device Security
    ble_device
        .security()
        .set_auth(AuthReq::all())
        .set_passkey(123456)
        .set_io_cap(SecurityIOCap::DisplayOnly)
        .resolve_rpa();

    // Obtain handle for server
    let server = ble_device.get_server();

    // Define server connect behaviour
    server.on_connect(|server, clntdesc| {
        // Print connected client data
        println!("{:?}", clntdesc);
        // Update connection parameters
        server
            .update_conn_params(clntdesc.conn_handle(), 24, 48, 0, 60)
            .unwrap();
    });

    // Define server disconnect behaviour
    server.on_disconnect(|_desc, _reason| {
        println!("Disconnected, back to advertising");
    });

    // Create a service with custom UUID
    let my_service = server.create_service(uuid128!("9b574847-f706-436c-bed7-fc01eb0965c1"));

    // Create a characteristic to associate with created service
    let my_service_characteristic = my_service.lock().create_characteristic(
        uuid128!("681285a6-247f-48c6-80ad-68c3dce18585"),
        NimbleProperties::WRITE | NimbleProperties::WRITE_ENC
    );

    let mut buf : [u8; 32] = Default::default();
    let init_value = match nvs_hander.get_raw("KEY_NUM",&mut buf )? {
        None => b"start value",
        Some(value) => value,
    };
    log::info!("init_value {:?}" ,init_value);
    // Modify characteristic value
    my_service_characteristic.lock().set_value(init_value);
    my_service_characteristic.lock().on_write(move |value|{
        nvs_hander.set_raw("KEY_NUM", value.recv_data()).unwrap();
        log::info!("current {:?}" ,value.current_data());
        log::info!("recv {:?}" ,value.recv_data());
    });

    // Configure Advertiser Data
    ble_advertiser
        .lock()
        .set_data(
            BLEAdvertisementData::new()
                .name("ESP32 Server")
                .add_service_uuid(uuid128!("9b574847-f706-436c-bed7-fc01eb0965c1")),
        )
        .unwrap();

    // Start Advertising
    ble_advertiser.lock().start().unwrap();
    return Ok(Bluetooth{
        characteristic: my_service_characteristic
    })
}