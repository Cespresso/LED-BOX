package com.example.ledboxdebug

import android.annotation.SuppressLint
import android.app.Application
import android.bluetooth.BluetoothDevice
import android.bluetooth.BluetoothGatt
import android.bluetooth.BluetoothGattCallback
import android.bluetooth.BluetoothGattCharacteristic
import android.bluetooth.BluetoothGattDescriptor
import android.bluetooth.BluetoothManager
import android.bluetooth.BluetoothProfile
import android.bluetooth.le.ScanCallback
import android.bluetooth.le.ScanFilter
import android.bluetooth.le.ScanResult
import android.bluetooth.le.ScanSettings
import android.util.Log
import androidx.lifecycle.AndroidViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.UUID

enum class ConnectionState {
    Disconnected, Scanning, Connecting, Connected, Ready
}

@Suppress("DEPRECATION")
class LedBoxViewModel(application: Application) : AndroidViewModel(application) {

    companion object {
        private const val TAG = "LedBoxDebug"
        private const val DEVICE_NAME = "LED BOX"
        val SERVICE_UUID: UUID = UUID.fromString("455aa9f0-2999-43de-81b4-54e0de255927")
        val MODE_UUID: UUID = UUID.fromString("681285a6-247f-48c6-80ad-68c3dce18586")
        val DISPLAY_UUID: UUID = UUID.fromString("681285a6-247f-48c6-80ad-68c3dce18585")
        val CCCD_UUID: UUID = UUID.fromString("00002902-0000-1000-8000-00805f9b34fb")

        val MODE_NAMES = mapOf(
            0 to "Pet",
            1 to "Pomodoro",
            2 to "Tools",
            3 to "Notification",
            4 to "SmartHome",
            5 to "Monitor",
        )
    }

    private val timeFormat = SimpleDateFormat("HH:mm:ss.SSS", Locale.getDefault())

    private val _connectionState = MutableStateFlow(ConnectionState.Disconnected)
    val connectionState = _connectionState.asStateFlow()

    private val _currentMode = MutableStateFlow<Int?>(null)
    val currentMode = _currentMode.asStateFlow()

    private val _displayData = MutableStateFlow(IntArray(8))
    val displayData = _displayData.asStateFlow()

    private val _logs = MutableStateFlow<List<String>>(emptyList())
    val logs = _logs.asStateFlow()

    private val bluetoothManager =
        application.getSystemService(BluetoothManager::class.java)
    private val bluetoothAdapter = bluetoothManager.adapter
    private var gatt: BluetoothGatt? = null
    private var modeCharacteristic: BluetoothGattCharacteristic? = null
    private var displayCharacteristic: BluetoothGattCharacteristic? = null

    private fun log(message: String) {
        val timestamp = timeFormat.format(Date())
        val entry = "[$timestamp] $message"
        Log.d(TAG, message)
        _logs.value = _logs.value + entry
    }

    // --- Public actions ---

    @SuppressLint("MissingPermission")
    fun startScan() {
        if (_connectionState.value != ConnectionState.Disconnected) return

        val scanner = bluetoothAdapter?.bluetoothLeScanner
        if (scanner == null) {
            log("BLE Scanner not available")
            return
        }

        _connectionState.value = ConnectionState.Scanning
        log("Scanning for '$DEVICE_NAME'...")

        val filters = listOf(
            ScanFilter.Builder().setDeviceName(DEVICE_NAME).build()
        )
        val settings = ScanSettings.Builder()
            .setScanMode(ScanSettings.SCAN_MODE_LOW_LATENCY)
            .build()

        scanner.startScan(filters, settings, scanCallback)
    }

    @SuppressLint("MissingPermission")
    fun disconnect() {
        bluetoothAdapter?.bluetoothLeScanner?.stopScan(scanCallback)
        gatt?.disconnect()
        gatt?.close()
        gatt = null
        modeCharacteristic = null
        displayCharacteristic = null
        _connectionState.value = ConnectionState.Disconnected
        _currentMode.value = null
        log("Disconnected")
    }

    @SuppressLint("MissingPermission")
    fun writeMode(mode: Int) {
        val char = modeCharacteristic ?: run {
            log("Mode characteristic not available")
            return
        }
        char.value = byteArrayOf(mode.toByte())
        val success = gatt?.writeCharacteristic(char) ?: false
        log("Write mode=$mode (${MODE_NAMES[mode]}): ${if (success) "sent" else "failed"}")
    }

    @SuppressLint("MissingPermission")
    fun writeDisplayData(data: IntArray) {
        val char = displayCharacteristic ?: run {
            log("Display characteristic not available")
            return
        }
        char.value = ByteArray(8) { data[it].toByte() }
        val success = gatt?.writeCharacteristic(char) ?: false
        log("Write display: ${if (success) "sent" else "failed"}")
    }

    @SuppressLint("MissingPermission")
    fun readMode() {
        val char = modeCharacteristic ?: return
        gatt?.readCharacteristic(char)
        log("Reading mode...")
    }

    @SuppressLint("MissingPermission")
    fun readDisplayData() {
        val char = displayCharacteristic ?: return
        gatt?.readCharacteristic(char)
        log("Reading display data...")
    }

    fun togglePixel(row: Int, col: Int) {
        val data = _displayData.value.copyOf()
        data[row] = data[row] xor (1 shl (7 - col))
        _displayData.value = data
    }

    fun clearDisplay() {
        _displayData.value = IntArray(8)
    }

    // --- BLE Callbacks ---

    private val scanCallback = object : ScanCallback() {
        @SuppressLint("MissingPermission")
        override fun onScanResult(callbackType: Int, result: ScanResult) {
            bluetoothAdapter?.bluetoothLeScanner?.stopScan(this)
            _connectionState.value = ConnectionState.Connecting
            log("Found: ${result.device.address} (RSSI=${result.rssi})")

            result.device.connectGatt(
                getApplication(),
                false,
                gattCallback,
                BluetoothDevice.TRANSPORT_LE
            )
        }

        override fun onScanFailed(errorCode: Int) {
            _connectionState.value = ConnectionState.Disconnected
            log("Scan failed (error=$errorCode)")
        }
    }

    private val gattCallback = object : BluetoothGattCallback() {

        @SuppressLint("MissingPermission")
        override fun onConnectionStateChange(gatt: BluetoothGatt, status: Int, newState: Int) {
            when (newState) {
                BluetoothProfile.STATE_CONNECTED -> {
                    this@LedBoxViewModel.gatt = gatt
                    _connectionState.value = ConnectionState.Connected
                    log("Connected, discovering services...")
                    gatt.discoverServices()
                }
                BluetoothProfile.STATE_DISCONNECTED -> {
                    _connectionState.value = ConnectionState.Disconnected
                    log("Connection lost (status=$status)")
                    gatt.close()
                    this@LedBoxViewModel.gatt = null
                }
            }
        }

        @SuppressLint("MissingPermission")
        override fun onServicesDiscovered(gatt: BluetoothGatt, status: Int) {
            if (status != BluetoothGatt.GATT_SUCCESS) {
                log("Service discovery failed (status=$status)")
                return
            }

            val service = gatt.getService(SERVICE_UUID)
            if (service == null) {
                log("Service NOT found! Check UUID.")
                return
            }
            log("Service found")

            modeCharacteristic = service.getCharacteristic(MODE_UUID)
            displayCharacteristic = service.getCharacteristic(DISPLAY_UUID)

            if (modeCharacteristic != null) {
                log("Mode characteristic found (props=${modeCharacteristic!!.properties})")
            } else {
                log("Mode characteristic NOT found!")
            }

            if (displayCharacteristic != null) {
                log("Display characteristic found (props=${displayCharacteristic!!.properties})")
            } else {
                log("Display characteristic NOT found!")
            }

            // Enable notifications on mode characteristic
            modeCharacteristic?.let { char ->
                gatt.setCharacteristicNotification(char, true)
                val descriptor = char.getDescriptor(CCCD_UUID)
                if (descriptor != null) {
                    descriptor.value = BluetoothGattDescriptor.ENABLE_NOTIFICATION_VALUE
                    gatt.writeDescriptor(descriptor)
                    log("Enabling mode notifications...")
                } else {
                    log("CCCD descriptor not found on mode characteristic")
                }
            }

            _connectionState.value = ConnectionState.Ready
            log("Ready")
        }

        override fun onDescriptorWrite(
            gatt: BluetoothGatt,
            descriptor: BluetoothGattDescriptor,
            status: Int
        ) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                log("Notification enabled")
                // Read initial mode after notifications are set up
                readMode()
            } else {
                log("Descriptor write failed (status=$status)")
            }
        }

        override fun onCharacteristicRead(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            status: Int
        ) {
            if (status != BluetoothGatt.GATT_SUCCESS) {
                log("Read failed (status=$status)")
                return
            }
            val value = characteristic.value ?: return
            when (characteristic.uuid) {
                MODE_UUID -> {
                    val mode = value[0].toInt() and 0xFF
                    _currentMode.value = mode
                    log("Mode = $mode (${MODE_NAMES[mode] ?: "unknown"})")
                }
                DISPLAY_UUID -> {
                    val data = IntArray(8) { if (it < value.size) value[it].toInt() and 0xFF else 0 }
                    _displayData.value = data
                    log("Display = ${data.joinToString(" ") { "%02X".format(it) }}")
                }
            }
        }

        override fun onCharacteristicWrite(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic,
            status: Int
        ) {
            if (status == BluetoothGatt.GATT_SUCCESS) {
                log("Write OK (${characteristic.uuid.toString().takeLast(4)})")
                if (characteristic.uuid == MODE_UUID) {
                    readMode()
                }
            } else {
                log("Write FAILED (status=$status)")
            }
        }

        override fun onCharacteristicChanged(
            gatt: BluetoothGatt,
            characteristic: BluetoothGattCharacteristic
        ) {
            if (characteristic.uuid == MODE_UUID) {
                val value = characteristic.value ?: return
                val mode = value[0].toInt() and 0xFF
                _currentMode.value = mode
                log("Mode changed -> $mode (${MODE_NAMES[mode] ?: "unknown"})")
            }
        }
    }

    override fun onCleared() {
        @SuppressLint("MissingPermission")
        fun cleanup() {
            gatt?.disconnect()
            gatt?.close()
        }
        cleanup()
    }
}
