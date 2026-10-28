package com.sanchezfauste.fausteplayer

import android.content.Context
import android.database.ContentObserver
import android.os.Handler
import android.os.Looper
import android.provider.MediaStore
import android.util.Log
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Hands the device's audio files to the media browser (mobile spec MB7):
 * asks MediaStore on a background thread and passes the paths through
 * [Native.setMediaFiles]. Music is only read (spec §2).
 */
class MediaFiles(private val context: Context) {
    private val worker = Executors.newSingleThreadExecutor()

    /** A query is queued and not started: later refreshes join it. */
    private val queued = AtomicBoolean(false)
    private val observer = object : ContentObserver(Handler(Looper.getMainLooper())) {
        override fun onChange(selfChange: Boolean) = refresh()
    }

    /** Follows MediaStore's changes (a scan finishing) until [stop]. */
    fun start() {
        context.contentResolver.registerContentObserver(
            MediaStore.Audio.Media.EXTERNAL_CONTENT_URI, true, observer
        )
        refresh()
    }

    fun stop() {
        context.contentResolver.unregisterContentObserver(observer)
        worker.shutdown()
    }

    /**
     * Lists the files again; without permission the list is empty. A scan
     * notifies once per file, so refreshes asked while a query waits to
     * start are one query.
     */
    fun refresh() {
        if (worker.isShutdown || !queued.compareAndSet(false, true)) {
            return
        }
        worker.execute {
            queued.set(false)
            val paths = try {
                query()
            } catch (e: SecurityException) {
                emptyList()
            } catch (e: RuntimeException) {
                Log.w(TAG, "could not list the audio files", e)
                emptyList()
            }
            Native.setMediaFiles(paths.toTypedArray())
        }
    }

    /**
     * One record per file: the path, then the title, artist, album and
     * duration in ms, NUL-separated (`MediaFile::from_record`); a missing
     * tag is an empty field.
     */
    @Suppress("DEPRECATION") // DATA is the path the decoders open.
    private fun query(): List<String> {
        val columns = arrayOf(
            MediaStore.Audio.Media.DATA,
            MediaStore.Audio.Media.TITLE,
            MediaStore.Audio.Media.ARTIST,
            MediaStore.Audio.Media.ALBUM,
            MediaStore.Audio.Media.DURATION,
        )
        val records = mutableListOf<String>()
        context.contentResolver.query(
            MediaStore.Audio.Media.EXTERNAL_CONTENT_URI, columns, null, null, null
        )?.use { cursor ->
            val index = columns.map(cursor::getColumnIndexOrThrow)
            while (cursor.moveToNext()) {
                val path = cursor.getString(index[0]) ?: continue
                val tags = index.drop(1).map { column ->
                    // MediaStore's "<unknown>" artist is no tag; a NUL would
                    // split the record.
                    cursor.getString(column)
                        ?.takeUnless { it == MediaStore.UNKNOWN_STRING }
                        ?.replace('\u0000', ' ')
                        .orEmpty()
                }
                records.add((listOf(path) + tags).joinToString("\u0000"))
            }
        }
        return records
    }

    private companion object {
        const val TAG = "fauste"
    }
}
