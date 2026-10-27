package com.sanchezfauste.fausteplayer

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Intent
import android.content.pm.ServiceInfo
import android.media.AudioAttributes
import android.media.AudioFocusRequest
import android.media.AudioManager
import android.media.session.MediaSession
import android.media.session.PlaybackState
import android.os.Handler
import android.os.IBinder
import android.os.Looper

/**
 * Keeps the process in the foreground while playout is active (MB1). It
 * offers no transport controls and ignores media buttons (MB4b); a lost
 * audio focus is only reported, never acted on (MB4). Not sticky: a killed
 * process starts cold and plays nothing (MB3).
 */
class PlaybackService : Service() {
    private val handler = Handler(Looper.getMainLooper())
    private var session: MediaSession? = null
    private var focus: AudioFocusRequest? = null
    private val refresh = object : Runnable {
        override fun run() {
            val state = Native.state()
            if (state and Native.PLAYOUT_ACTIVE == 0) {
                stopSelf()
                return
            }
            followFocus(state)
            notificationManager().notify(NOTIFICATION, notification())
            handler.postDelayed(this, REFRESH_MS)
        }
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        running = true
        notificationManager().createNotificationChannel(
            NotificationChannel(
                CHANNEL,
                Native.notificationChannelName(),
                NotificationManager.IMPORTANCE_LOW,
            )
        )
        session = MediaSession(this, "FaustePlayer").apply {
            setPlaybackState(PlaybackState.Builder().setActions(0).build())
            setCallback(object : MediaSession.Callback() {
                override fun onMediaButtonEvent(mediaButtonIntent: Intent): Boolean = true
            })
            isActive = true
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForeground(
            NOTIFICATION,
            notification(),
            ServiceInfo.FOREGROUND_SERVICE_TYPE_MEDIA_PLAYBACK,
        )
        followFocus(Native.state())
        handler.removeCallbacks(refresh)
        handler.postDelayed(refresh, REFRESH_MS)
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        handler.removeCallbacks(refresh)
        abandonFocus()
        session?.release()
        session = null
        running = false
        super.onDestroy()
    }

    /**
     * Holds the audio focus only while something sounds: a loaded track
     * alone must not pause the operator's other apps.
     */
    private fun followFocus(state: Int) {
        if (state and Native.SOUNDING != 0) {
            if (focus == null) requestFocus()
        } else {
            abandonFocus()
        }
    }

    private fun abandonFocus() {
        focus?.let { getSystemService(AudioManager::class.java).abandonAudioFocusRequest(it) }
        focus = null
        Native.setAudioFocusLost(false)
    }

    /** The system does not duck us (we "pause when ducked", but never do). */
    private fun requestFocus() {
        val attributes = AudioAttributes.Builder()
            .setUsage(AudioAttributes.USAGE_MEDIA)
            .setContentType(AudioAttributes.CONTENT_TYPE_MUSIC)
            .build()
        val request = AudioFocusRequest.Builder(AudioManager.AUDIOFOCUS_GAIN)
            .setAudioAttributes(attributes)
            .setWillPauseWhenDucked(true)
            .setOnAudioFocusChangeListener { change -> Native.setAudioFocusLost(change < 0) }
            .build()
        focus = request
        val granted = getSystemService(AudioManager::class.java).requestAudioFocus(request)
        Native.setAudioFocusLost(granted != AudioManager.AUDIOFOCUS_REQUEST_GRANTED)
    }

    private fun notification(): Notification {
        val open = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT,
        )
        val text = Native.onAirText()
        return Notification.Builder(this, CHANNEL)
            .setContentTitle(getString(R.string.app_name))
            .setContentText(text.lineSequence().firstOrNull().orEmpty())
            .setStyle(Notification.BigTextStyle().bigText(text))
            .setSmallIcon(R.drawable.ic_notification)
            .setContentIntent(open)
            .setOngoing(true)
            .setOnlyAlertOnce(true)
            .build()
    }

    private fun notificationManager() = getSystemService(NotificationManager::class.java)

    companion object {
        /** Whether the service exists, so the activity starts it only once. */
        @Volatile
        var running = false
            private set

        private const val CHANNEL = "playout"
        private const val NOTIFICATION = 1
        private const val REFRESH_MS = 2_000L
    }
}
