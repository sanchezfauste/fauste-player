package com.sanchezfauste.fausteplayer

import android.Manifest
import android.content.ActivityNotFoundException
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.Log
import android.view.KeyEvent
import android.view.WindowManager
import androidx.activity.OnBackPressedCallback
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import com.google.androidgamesdk.GameActivity

/**
 * The interface runs in Rust (`android_main` in crates/fp-mobile). This
 * activity asks for permissions, reports the insets and the device's
 * music, starts the playout service while playout is active and keeps the
 * screen on (mobile spec §5).
 */
class MainActivity : GameActivity() {
    private val handler = Handler(Looper.getMainLooper())
    private val media by lazy { MediaFiles(applicationContext) }
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
        media.start()
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                moveTaskToBack(true)
            }
        })
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
        // Files copied while away, or access granted in the system settings.
        media.refresh()
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray,
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == PERMISSIONS) {
            media.refresh()
        }
    }

    override fun onDestroy() {
        media.stop()
        super.onDestroy()
    }

    /** Once more on the way out: Play then Home within a poll still gets the service. */
    override fun onPause() {
        follow(Native.state())
        handler.removeCallbacks(poll)
        super.onPause()
    }

    /** Opens a link from the interface (the About window); called from Rust. */
    @Suppress("unused")
    fun openUrl(url: String) {
        runOnUiThread {
            try {
                startActivity(
                    Intent(Intent.ACTION_VIEW, Uri.parse(url))
                        .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                )
            } catch (e: ActivityNotFoundException) {
                Log.w("fauste", "no app opens $url", e)
            } catch (e: RuntimeException) {
                // On the UI thread: a crash here would end the playout too.
                Log.w("fauste", "could not open $url", e)
            }
        }
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

    /**
     * Back never destroys the activity: the playout keeps its interface.
     * GameActivity hands every key to the native side, which consumes Back,
     * so the key is taken here before it; the gesture reaches the callback
     * registered in `onCreate`.
     */
    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean =
        keyCode == KeyEvent.KEYCODE_BACK || super.onKeyDown(keyCode, event)

    override fun onKeyUp(keyCode: Int, event: KeyEvent): Boolean {
        if (keyCode == KeyEvent.KEYCODE_BACK) {
            moveTaskToBack(true)
            return true
        }
        return super.onKeyUp(keyCode, event)
    }

    private companion object {
        const val POLL_MS = 500L
        const val PERMISSIONS = 1
    }
}
