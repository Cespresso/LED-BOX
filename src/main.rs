use esp32_nimble::{BLEAdvertisementData, BLEDevice, NimbleProperties, uuid128};
use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};

mod utils;
fn main() -> Result<(), Box<dyn std::error::Error>>  {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Start LED BOX");
    // Setup handler for device peripherals
    let peripherals = Peripherals::take().unwrap();
    // Create handles for SPI pins
    let mut spi = utils::led::initialize_spi(peripherals);
    utils::led::initialize_matrix_display(&mut spi);

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
    let my_service = server.create_service(uuid128!("455aa9f0-2999-43de-81b4-54e0de255927"));

    // Create a characteristic to associate with created service
    let my_service_characteristic = my_service.lock().create_characteristic(
        uuid128!("681285a6-247f-48c6-80ad-68c3dce18585"),
        NimbleProperties::WRITE | NimbleProperties::WRITE_ENC | NimbleProperties::READ | NimbleProperties::READ_ENC
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
                .name("LED BOX")
                .add_service_uuid(uuid128!("455aa9f0-2999-43de-81b4-54e0de255927")),
        )
        .unwrap();

    // Start Advertising
    ble_advertiser.lock().start().unwrap();

    loop {
        FreeRtos::delay_ms(2000_u32);
        let mut ch = my_service_characteristic.lock();
        let matrix = ch.value_mut().as_slice();
        if matrix.len() == 8 {
            for addr in 1..9 {
                spi.write(&[addr, matrix[(addr as usize)-1]]).unwrap();
            }
        }else{
            spi.write(&[1, 0x00]).unwrap();
            spi.write(&[2, 0x42]).unwrap();
            spi.write(&[3, 0x24]).unwrap();
            spi.write(&[4, 0x18]).unwrap();
            spi.write(&[5, 0x18]).unwrap();
            spi.write(&[6, 0x24]).unwrap();
            spi.write(&[7, 0x42]).unwrap();
            spi.write(&[8, 0x00]).unwrap();
        }
    }
}
