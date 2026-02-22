use std::sync::Arc;

use esp32_nimble::enums::{AuthReq, SecurityIOCap};
use esp32_nimble::utilities::mutex::Mutex;
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};

/// BLE protocol commands parsed from incoming data.
///
/// | Byte 0 | Payload              | Action              |
/// |--------|----------------------|---------------------|
/// | 0x01   | Byte 1: mode (0-5)   | Switch mode         |
/// | 0x02   | Bytes 1-8: 8 bytes   | Set LED display data|
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
}

impl BluetoothManager {
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
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

        // Parse BLE protocol on write
        let state_clone = state.clone();
        characteristic.lock().on_write(move |value| {
            let data = value.recv_data();
            log::info!("BLE recv: {:?}", data);

            if data.is_empty() {
                log::warn!("BLE: empty payload, ignoring");
                return;
            }

            let mut state = state_clone.lock();
            match data[0] {
                0x01 => {
                    // SwitchMode: expects 1 byte payload
                    if data.len() >= 2 {
                        let mode = data[1];
                        log::info!("BLE cmd: SwitchMode({})", mode);
                        state.pending_command = Some(BleCommand::SwitchMode(mode));
                    } else {
                        log::warn!("BLE: SwitchMode missing mode byte");
                    }
                }
                0x02 => {
                    // SetDisplayData: expects 8 bytes payload
                    if data.len() >= 9 {
                        let mut buf = [0u8; 8];
                        buf.copy_from_slice(&data[1..9]);
                        log::info!("BLE cmd: SetDisplayData({:?})", buf);
                        state.display_data = buf;
                        state.pending_command = Some(BleCommand::SetDisplayData(buf));
                    } else {
                        log::warn!(
                            "BLE: SetDisplayData needs 9 bytes, got {}",
                            data.len()
                        );
                    }
                }
                cmd => {
                    log::warn!("BLE: unknown command 0x{:02X}", cmd);
                }
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

        Ok(Self { state })
    }

    /// Take the pending command (if any), clearing it from the state.
    pub fn take_command(&self) -> Option<BleCommand> {
        self.state.lock().pending_command.take()
    }

    /// Get current display data (8 bytes for 8x8 LED matrix).
    pub fn get_display_data(&self) -> [u8; 8] {
        self.state.lock().display_data
    }
}
