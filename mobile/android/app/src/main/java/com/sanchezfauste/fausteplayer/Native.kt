package com.sanchezfauste.fausteplayer

/** The calls into the Rust library (crates/fp-mobile/src/jni.rs). None blocks. */
object Native {
    init {
        System.loadLibrary("fp_mobile")
    }

    /** Bit of [state]: playout is active, run the foreground service (MB1). */
    const val PLAYOUT_ACTIVE = 1

    /** Bit of [state]: keep the screen on (MB8). */
    const val KEEP_SCREEN_ON = 2

    /** Bit of [state]: something sounds; hold the audio focus only then (MB4). */
    const val SOUNDING = 4

    /** The system bars and display cutout, in physical pixels (MB10). */
    external fun setInsets(left: Int, top: Int, right: Int, bottom: Int)

    /** Another app took (or gave back) the audio focus; nothing pauses (MB4). */
    external fun setAudioFocusLost(lost: Boolean)

    /** The device's audio files, as paths the decoders open (MB7). */
    external fun setMediaFiles(paths: Array<String>)

    /** [PLAYOUT_ACTIVE] and [KEEP_SCREEN_ON] bits from the session's model. */
    external fun state(): Int

    /** What is on air, one line per player or cart; empty when nothing. */
    external fun onAirText(): String

    /** The notification channel's name, in the interface language. */
    external fun notificationChannelName(): String
}
