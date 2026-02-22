use std::sync::Arc;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::utilities::mutex::Mutex;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLECharacteristic, BLEDevice, NimbleProperties};

/// BLE commands queued for the main loop.
pub enum BleCommand {
    SwitchMode(u8),
    SetDisplayData([u8; 8]),
}

struct BleState {
    pending_command: Option<BleCommand>,
    display_data: [u8; 8],
}

pub struct BluetoothManager {
    state: Arc<Mutex<BleState>>,
    mode_characteristic: Arc<Mutex<BLECharacteristic>>,
}

impl BluetoothManager {
    pub fn init(initial_mode: u8) -> Result<Self, Box<dyn std::error::Error>> {
        let state = Arc::new(Mutex::new(BleState {
            pending_command: None,
            display_data: [0u8; 8],
        }));

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

        let service =
            server.create_service(uuid128!("455aa9f0-2999-43de-81b4-54e0de255927"));

        // --- Mode Characteristic (READ | WRITE | NOTIFY) ---
        let mode_characteristic = service.lock().create_characteristic(
            uuid128!("681285a6-247f-48c6-80ad-68c3dce18586"),
            NimbleProperties::READ
                | NimbleProperties::READ_ENC
                | NimbleProperties::WRITE
                | NimbleProperties::WRITE_ENC
                | NimbleProperties::NOTIFY,
        );
        mode_characteristic
            .lock()
            .set_value(&[initial_mode]);

        let state_clone = state.clone();
        mode_characteristic.lock().on_write(move |value| {
            let data = value.recv_data();
            log::info!("BLE mode write: {:?}", data);

            if data.is_empty() {
                log::warn!("BLE: mode write empty, ignoring");
                return;
            }

            let mode = data[0];
            if mode <= 5 {
                log::info!("BLE cmd: SwitchMode({})", mode);
                state_clone.lock().pending_command = Some(BleCommand::SwitchMode(mode));
            } else {
                log::warn!("BLE: invalid mode value 0x{:02X}", mode);
            }
        });

        // --- Display Data Characteristic (READ | WRITE) ---
        let display_characteristic = service.lock().create_characteristic(
            uuid128!("681285a6-247f-48c6-80ad-68c3dce18585"),
            NimbleProperties::READ
                | NimbleProperties::READ_ENC
                | NimbleProperties::WRITE
                | NimbleProperties::WRITE_ENC,
        );

        let state_clone = state.clone();
        display_characteristic.lock().on_write(move |value| {
            let data = value.recv_data();
            log::info!("BLE display write: {:?}", data);

            if data.len() >= 8 {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&data[..8]);
                log::info!("BLE cmd: SetDisplayData({:?})", buf);
                let mut state = state_clone.lock();
                state.display_data = buf;
                state.pending_command = Some(BleCommand::SetDisplayData(buf));
            } else {
                log::warn!("BLE: display data needs 8 bytes, got {}", data.len());
            }
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

        Ok(Self {
            state,
            mode_characteristic,
        })
    }

    /// Take the pending command (if any), clearing it from the state.
    pub fn take_command(&self) -> Option<BleCommand> {
        self.state.lock().pending_command.take()
    }

    /// Get current display data (8 bytes for 8x8 LED matrix).
    pub fn get_display_data(&self) -> [u8; 8] {
        self.state.lock().display_data
    }

    /// Update the Mode Characteristic value and notify connected clients.
    /// Call this when mode changes via button press or other non-BLE source.
    pub fn notify_mode_change(&self, mode: u8) {
        self.mode_characteristic
            .lock()
            .set_value(&[mode])
            .notify();
    }
}
