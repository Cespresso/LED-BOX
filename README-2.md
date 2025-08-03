# LED BOX Duplicate Functionality - README-2

This document describes the duplicate functionality created with `-2` suffix as requested in issue.

## Overview

All major functionality has been duplicated with `-2` suffix to create a parallel implementation alongside the original code. This includes BLE services, LED control functions, data structures, and integration in the main application.

## Duplicated Components

### 1. BLE Services (-2)

**Original Service:**
- Service UUID: `455aa9f0-2999-43de-81b4-54e0de255927`
- Characteristic UUID: `681285a6-247f-48c6-80ad-68c3dce18585`
- NVS Key: `KEY_NUM`
- Device Name: `LED BOX`

**Duplicate Service (-2):**
- Service UUID: `455aa9f0-2999-43de-81b4-54e0de255928`
- Characteristic UUID: `681285a6-247f-48c6-80ad-68c3dce18586`
- NVS Key: `KEY_NUM_2`
- Device Name: `LED BOX-2`

### 2. LED Control Functions (-2)

**Original Functions:**
- `initialize_spi()`
- `initialize_matrix_display()`

**Duplicate Functions (-2):**
- `initialize_spi_2()`
- `initialize_matrix_display_2()`

### 3. Bluetooth Module (-2)

**Original:**
- `Bluetooth` struct
- `init()` function

**Duplicate (-2):**
- `Bluetooth2` struct  
- `init_2()` function

### 4. Data Structures (-2)

**New Module: `data_2.rs`**
- `LEDMatrix` and `LEDMatrix2` structs
- `BLEConfig` and `BLEConfig2` structs
- Different default patterns and configurations

### 5. Main Application Integration

The main application now alternates between both services:
- First service displays for 2 seconds
- Second service (-2) displays for 2 seconds
- Different default LED patterns for visual distinction:
  - Original: Smiley face pattern
  - Duplicate (-2): Border/frame pattern

## Usage

Both services are automatically started and advertised simultaneously. Clients can connect to either:
- `LED BOX` (original service)
- `LED BOX-2` (duplicate service)

Each service maintains independent:
- NVS storage
- LED display patterns
- Connection handling
- Characteristic values

## Implementation Notes

- All duplicate functions maintain identical core functionality
- Different UUIDs ensure no conflicts between services
- Separate NVS namespaces prevent data collision
- Mutex handling ensures thread safety
- Resource sharing is properly managed

This implementation demonstrates a complete duplication of the LED BOX functionality while preserving the original code unchanged.