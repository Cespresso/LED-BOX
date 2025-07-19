# LED-BOX Project Guidelines

## Project Overview

LED-BOX is an ESP32-based IoT device that controls an 8x8 LED matrix display. The device can be controlled via Bluetooth Low Energy (BLE) and can also read data from NFC/RFID tags using an MFRC522 module. The project is written in Rust using the ESP-IDF framework.

### Key Features

- 8x8 LED matrix display control via SPI (MAX7219 driver)
- Bluetooth Low Energy (BLE) connectivity using the NimBLE stack
- NFC/RFID tag reading capability using MFRC522 module
- Non-volatile storage (NVS) for persisting settings and patterns

## Project Structure

```
LED-BOX/
├── Cargo.toml          # Project dependencies and configuration
├── Cargo.lock          # Locked dependencies
├── build.rs            # Build script for ESP-IDF integration
├── sdkconfig.defaults  # ESP-IDF configuration defaults
├── src/                # Main source code
│   ├── main.rs         # Application entry point
│   └── utils/          # Utility modules
│       ├── led.rs      # LED matrix control functions
│       └── bluetooth.rs # Bluetooth functionality
├── examples/           # Example applications
│   └── nfc.rs          # Example for NFC/RFID functionality
└── Readme.md           # Brief build instructions
```

## Build Instructions

The project uses the standard Rust toolchain with ESP-IDF support. To build and run the project:

1. **Setup Environment**:
   - Install Rust and the ESP-IDF toolchain
   - Install the `espup` tool: `cargo install espup`
   - Install the ESP-IDF environment: `espup install`
   - Activate the ESP-IDF environment: `. $HOME/export-esp.sh`

2. **Build and Flash the Main Application**:
   ```
   CRATE_CC_NO_DEFAULTS=1 cargo build
   ```

3. **Run Examples**:
   ```
   CRATE_CC_NO_DEFAULTS=1 cargo run --example nfc
   ```

## Hardware Requirements

- ESP32 development board
- MAX7219-based 8x8 LED matrix display
- MFRC522 RFID/NFC reader module (for NFC functionality)
- Appropriate connections:
  - SPI for LED matrix: GPIO8 (SCLK), GPIO9 (CS), GPIO10 (MOSI)
  - SPI for MFRC522 (example): GPIO8 (SCLK), GPIO20 (CS), GPIO9 (MOSI), GPIO10 (MISO)

## Code Style Guidelines

- Follow standard Rust coding conventions
- Use meaningful variable and function names
- Add comments for complex logic
- Keep functions focused on a single responsibility
- Use the Rust type system to prevent errors

## Testing

The project doesn't use the standard Rust test harness (disabled in Cargo.toml). Testing is primarily done by flashing the firmware to the device and verifying functionality manually.

## Bluetooth Functionality

The device advertises itself as "LED BOX" and provides a BLE service with a writable characteristic. Writing to this characteristic updates the pattern displayed on the LED matrix. The last pattern is stored in non-volatile storage and persists across power cycles.

## NFC/RFID Functionality

The device can read data from NFC tags using the MFRC522 module. The example in `examples/nfc.rs` demonstrates how to read NDEF-formatted data from NFC tags.

## Development Workflow

1. Make changes to the code
2. Build the project
3. Flash to the ESP32 device
4. Test functionality
5. Iterate as needed

## Troubleshooting

- If build fails, ensure the ESP-IDF environment is properly set up
- Check hardware connections if the device doesn't function as expected
- Use the serial console for debugging output
- The project uses the ESP-IDF logging facility, so check log output for errors