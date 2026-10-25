package com.sanchezfauste.fausteplayer

import android.Manifest
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.view.WindowManager
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import com.google.androidgamesdk.GameActivity

/**
 * The interface runs in Rust (`android_main` in crates/fp-mobile). This
 * activity asks for permissions, reports the insets, starts the playout
 * service while playout is active and keeps the screen on (mobile spec §5).
 */
class MainActivity : GameActivity() {
    private val handler = Handler(Looper.getMainLooper())
    private val poll = object : Runnable {
        override fun run() {
            follow(Native.state())
            handler.postDelayed(this, POLL_MS)
        }
    }

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
        requestPermissions(wanted.toTypedArray(), PERMISSIONS)
        ViewCompat.setOnApplyWindowInsetsListener(window.decorView) { view, insets ->
            val bars = insets.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            Native.setInsets(bars.left, bars.top, bars.right, bars.bottom)
            ViewCompat.onApplyWindowInsets(view, insets)
        }
    }

    /** Polled only while on screen: the service is started from the foreground. */
    override fun onResume() {
        super.onResume()
        handler.post(poll)
    }

    /** Once more on the way out: Play then Home within a poll still gets the service. */
    override fun onPause() {
        follow(Native.state())
        handler.removeCallbacks(poll)
        super.onPause()
    }

    private fun follow(state: Int) {
        if (state and Native.PLAYOUT_ACTIVE != 0 && !PlaybackService.running) {
            ContextCompat.startForegroundService(this, Intent(this, PlaybackService::class.java))
        }
        if (state and Native.KEEP_SCREEN_ON != 0) {
            window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        } else {
            window.clearFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)
        }
    }

    /** Back never destroys the activity: the playout keeps its interface. */
    @Deprecated("Back goes to the background")
    override fun onBackPressed() {
        moveTaskToBack(true)
    }

    private companion object {
        const val POLL_MS = 500L
        const val PERMISSIONS = 1
    }
}
