#!/usr/bin/env python3
"""
BLE notification sender for LED BOX.

Sends a notification command to the ESP32 LED BOX device via BLE.
Designed to be called from Claude Code hooks.

Usage:
    python3 ble-notify.py waiting    # Claude is waiting for input
    python3 ble-notify.py complete   # Claude finished responding

Requirements:
    pip install bleak
"""

import asyncio
import sys

from bleak import BleakClient, BleakScanner

DEVICE_NAME = "LED BOX"
DISPLAY_CHAR_UUID = "681285a6-247f-48c6-80ad-68c3dce18585"

NOTIFICATION_TYPES = {
    "waiting": 0x01,
    "complete": 0x02,
}


async def send_notification(notification_type: str) -> None:
    code = NOTIFICATION_TYPES.get(notification_type)
    if code is None:
        print(f"Unknown notification type: {notification_type}", file=sys.stderr)
        print(f"Valid types: {', '.join(NOTIFICATION_TYPES.keys())}", file=sys.stderr)
        sys.exit(1)

    # Scan for the device
    print(f"Scanning for '{DEVICE_NAME}'...", file=sys.stderr)
    device = await BleakScanner.find_device_by_name(DEVICE_NAME, timeout=5.0)
    if device is None:
        print(f"Device '{DEVICE_NAME}' not found", file=sys.stderr)
        sys.exit(1)

    print(f"Found {device.name} ({device.address})", file=sys.stderr)

    # Connect and write the notification command
    # Use 8 bytes (pad with zeros) to match the Display Data characteristic format
    data = bytes([code] + [0] * 7)

    async with BleakClient(device, timeout=10.0) as client:
        print(f"Connected. Sending notification: {notification_type} (0x{code:02X})", file=sys.stderr)
        await client.write_gatt_char(DISPLAY_CHAR_UUID, data)
        print("Notification sent successfully", file=sys.stderr)


def main() -> None:
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <{'|'.join(NOTIFICATION_TYPES.keys())}>", file=sys.stderr)
        sys.exit(1)

    asyncio.run(send_notification(sys.argv[1]))


if __name__ == "__main__":
    main()
