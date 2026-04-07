package com.example.ledboxdebug

import android.media.audiofx.Visualizer
import android.util.Log
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * Captures audio playback output using [Visualizer] and converts FFT data
 * into 8-band column heights (0-8) suitable for the 8x8 LED matrix.
 */
class AudioVisualizerManager {

    companion object {
        private const val TAG = "AudioVisualizer"
        private const val CAPTURE_SIZE = 128
        private const val NUM_BANDS = 8
        private const val MAX_HEIGHT = 8
    }

    private var visualizer: Visualizer? = null

    private val _columns = MutableStateFlow(IntArray(NUM_BANDS))
    val columns = _columns.asStateFlow()

    private val _isActive = MutableStateFlow(false)
    val isActive = _isActive.asStateFlow()

    /**
     * Start capturing audio playback output.
     * Requires RECORD_AUDIO and MODIFY_AUDIO_SETTINGS permissions.
     */
    fun start(): Boolean {
        if (visualizer != null) return true

        return try {
            val viz = Visualizer(0).apply {
                captureSize = CAPTURE_SIZE
                setDataCaptureListener(
                    object : Visualizer.OnDataCaptureListener {
                        override fun onWaveFormDataCapture(
                            visualizer: Visualizer,
                            waveform: ByteArray,
                            samplingRate: Int
                        ) {
                            // Not used — we use FFT for frequency bands
                        }

                        override fun onFftDataCapture(
                            visualizer: Visualizer,
                            fft: ByteArray,
                            samplingRate: Int
                        ) {
                            _columns.value = fftToBands(fft)
                        }
                    },
                    Visualizer.getMaxCaptureRate(),
                    false, // waveform disabled
                    true   // FFT enabled
                )
                enabled = true
            }
            visualizer = viz
            _isActive.value = true
            Log.d(TAG, "Started (captureSize=$CAPTURE_SIZE, rate=${Visualizer.getMaxCaptureRate()})")
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to start: ${e.message}")
            false
        }
    }

    fun stop() {
        visualizer?.let {
            it.enabled = false
            it.release()
        }
        visualizer = null
        _isActive.value = false
        _columns.value = IntArray(NUM_BANDS)
        Log.d(TAG, "Stopped")
    }

    /**
     * Convert FFT byte array to 8 frequency band magnitudes (0-8).
     *
     * The FFT output from Visualizer is interleaved real/imaginary pairs:
     *   fft[0] = DC real, fft[1] = Nyquist real,
     *   fft[2] = bin1 real, fft[3] = bin1 imag, ...
     *
     * With captureSize=128, we get 64 frequency bins (indices 0-63).
     * We group them into 8 bands with roughly logarithmic spacing to
     * emphasize lower frequencies where most musical content lives.
     */
    private fun fftToBands(fft: ByteArray): IntArray {
        val numBins = fft.size / 2
        val magnitudes = FloatArray(numBins)

        // Bin 0: DC component
        magnitudes[0] = kotlin.math.abs(fft[0].toFloat())
        // Bin numBins-1: Nyquist (stored in fft[1])
        if (numBins > 1) {
            magnitudes[numBins - 1] = kotlin.math.abs(fft[1].toFloat())
        }
        // Bins 1..numBins-2: complex pairs
        for (i in 1 until numBins - 1) {
            val real = fft[2 * i].toFloat()
            val imag = fft[2 * i + 1].toFloat()
            magnitudes[i] = kotlin.math.sqrt(real * real + imag * imag)
        }

        // Band boundaries focused on musically relevant range (~100Hz-8kHz).
        // With 44.1kHz/128 capture: each bin ≈ 345Hz.
        // Bins 1-24 cover ~345Hz-8.3kHz where most audio energy exists.
        val bandEdges = intArrayOf(1, 2, 3, 5, 7, 10, 14, 19, 25)

        val bands = IntArray(NUM_BANDS)
        for (band in 0 until NUM_BANDS) {
            val from = bandEdges[band]
            val to = bandEdges[band + 1]
            if (from >= numBins) break

            var sum = 0f
            var count = 0
            for (bin in from until minOf(to, numBins)) {
                sum += magnitudes[bin]
                count++
            }
            val avg = if (count > 0) sum / count else 0f

            // Map magnitude to 0-8 range
            // Typical magnitudes range 0-~80 for loud audio
            val normalized = (avg / 50f * MAX_HEIGHT).toInt().coerceIn(0, MAX_HEIGHT)
            bands[band] = normalized
        }

        return bands
    }
}
