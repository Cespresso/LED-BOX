use std::sync::Arc;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::utilities::mutex::Mutex;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLECharacteristic, BLEDevice, NimbleProperties};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs};

pub struct BluetoothManager {
    pub characteristic: Arc<Mutex<BLECharacteristic>>,
}

impl BluetoothManager {
    pub fn init(
        nvs_partition: EspDefaultNvsPartition,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut nvs = EspNvs::new(nvs_partition, "TEST", true)?;

        let ble_device = BLEDevice::take();
        let ble_advertiser = ble_device.get_advertising();

        // Configure Device Security
        ble_device
            .security()
            .set_auth(AuthReq::all())
            .set_passkey(123456)
            .set_io_cap(SecurityIOCap::DisplayOnly)
            .resolve_rpa();

        let server = ble_device.get_server();

        server.on_connect(|server, clntdesc| {
            log::info!("BLE client connected: {:?}", clntdesc);
            server
                .update_conn_params(clntdesc.conn_handle(), 24, 48, 0, 60)
                .unwrap();
        });

        server.on_disconnect(|_desc, _reason| {
            log::info!("BLE client disconnected");
        });

        // Create service and characteristic
        let service =
            server.create_service(uuid128!("455aa9f0-2999-43de-81b4-54e0de255927"));
        let characteristic = service.lock().create_characteristic(
            uuid128!("681285a6-247f-48c6-80ad-68c3dce18585"),
            NimbleProperties::READ
                | NimbleProperties::READ_ENC
                | NimbleProperties::WRITE
                | NimbleProperties::WRITE_ENC,
        );

        // Load initial value from NVS
        let mut buf: [u8; 32] = [0; 32];
        let init_value = match nvs.get_raw("KEY_NUM", &mut buf)? {
            Some(value) => value,
            None => b"start value",
        };
        log::info!("BLE init value: {:?}", init_value);
        characteristic.lock().set_value(init_value);

        // Save received data to NVS on write
        characteristic.lock().on_write(move |value| {
            nvs.set_raw("KEY_NUM", value.recv_data()).unwrap();
            log::info!("BLE recv: {:?}", value.recv_data());
        });

        // Configure and start advertising
        ble_advertiser
            .lock()
            .set_data(
                BLEAdvertisementData::new()
                    .name("LED BOX")
                    .add_service_uuid(uuid128!("455aa9f0-2999-43de-81b4-54e0de255927")),
            )
            .unwrap();
        ble_advertiser.lock().start().unwrap();
        log::info!("BLE advertising started as 'LED BOX'");

        Ok(Self { characteristic })
    }

    /// Get current display data from BLE characteristic
    pub fn get_display_data(&self) -> Vec<u8> {
        self.characteristic.lock().value_mut().as_slice().to_vec()
    }
}
