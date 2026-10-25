package com.sanchezfauste.faustespike

import android.Manifest
import android.content.Intent
import android.os.Build
import android.os.Bundle
import androidx.core.content.ContextCompat
import com.google.androidgamesdk.GameActivity

/** SPIKE: the egui UI runs in Rust (`android_main`); this only asks for
 *  permissions and starts the foreground service. */
class MainActivity : GameActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val wanted = buildList {
            if (Build.VERSION.SDK_INT >= 33) {
                add(Manifest.permission.READ_MEDIA_AUDIO)
                add(Manifest.permission.POST_NOTIFICATIONS)
            } else {
                add(Manifest.permission.READ_EXTERNAL_STORAGE)
            }
        }
        requestPermissions(wanted.toTypedArray(), 1)
        ContextCompat.startForegroundService(this, Intent(this, PlaybackService::class.java))
    }

    /** SPIKE: never destroy the activity on Back; the playout keeps its UI. */
    @Deprecated("Back goes to the background")
    override fun onBackPressed() {
        moveTaskToBack(true)
    }

    companion object {
        init { System.loadLibrary("fp_spike_android") }
    }
}
