package com.example.ledboxdebug

import android.app.Application
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import androidx.lifecycle.AndroidViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Thin ViewModel that binds to [LedBoxService] and proxies its state to the UI.
 * All BLE and audio logic lives in the Service.
 */
class LedBoxViewModel(application: Application) : AndroidViewModel(application) {

    private val _service = MutableStateFlow<LedBoxService?>(null)
    val service: StateFlow<LedBoxService?> = _service.asStateFlow()

    private val connection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, binder: IBinder?) {
            _service.value = (binder as LedBoxService.LocalBinder).service
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            _service.value = null
        }
    }

    init {
        // Start and bind to the service
        val ctx = getApplication<Application>()
        val intent = Intent(ctx, LedBoxService::class.java)
        ctx.startForegroundService(intent)
        ctx.bindService(intent, connection, Context.BIND_AUTO_CREATE)
    }

    override fun onCleared() {
        getApplication<Application>().unbindService(connection)
    }
}
