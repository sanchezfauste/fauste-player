//! The calls of the Kotlin class `com.sanchezfauste.fausteplayer.Native`
//! (each entry point only reads or stores a value and none blocks, except
//! `quit`, which the shell calls off its main thread), and the calls into
//! the activity, `open_url` and `leave`.

use std::sync::atomic::Ordering;

use jni::JNIEnv;
use jni::JavaVM;
use jni::objects::{JObject, JObjectArray, JString, JValueOwned};
use jni::sys::{jboolean, jint, jstring};

use fp_app::services::ServiceRequest;

use crate::android::{ACTIVITY, BACK, INSETS, REPAINT, SESSION};
use crate::platform::{AUDIO_FOCUS_LOST, MEDIA};
use crate::shell_state::{on_air_text, shell_state};

/// A Java string, or null when it cannot be made.
fn java_string(env: &mut JNIEnv<'_>, text: &str) -> jstring {
    env.new_string(text)
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[allow(unsafe_code)]
// SAFETY: a JNI entry point found by its mangled Java name; the signature
// matches `external fun setInsets(Int, Int, Int, Int)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_sanchezfauste_fausteplayer_Native_setInsets(
    _env: JNIEnv<'_>,
    _this: JObject<'_>,
    left: jint,
    top: jint,
    right: jint,
    bottom: jint,
) {
    for (slot, value) in INSETS.iter().zip([left, top, right, bottom]) {
        slot.store((value.max(0) as f32).to_bits(), Ordering::Relaxed);
    }
}

#[allow(unsafe_code)]
// SAFETY: as above; `external fun setAudioFocusLost(Boolean)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_sanchezfauste_fausteplayer_Native_setAudioFocusLost(
    _env: JNIEnv<'_>,
    _this: JObject<'_>,
    lost: jboolean,
) {
    AUDIO_FOCUS_LOST.store(lost != 0, Ordering::Relaxed);
}

#[allow(unsafe_code)]
// SAFETY: as above; `external fun state(): Int`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_sanchezfauste_fausteplayer_Native_state(
    _env: JNIEnv<'_>,
    _this: JObject<'_>,
) -> jint {
    SESSION
        .get()
        .map_or(0, |s| shell_state(&s.handle.model.load()))
}

#[allow(unsafe_code)]
// SAFETY: as above; `external fun onAirText(): String`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_sanchezfauste_fausteplayer_Native_onAirText(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
) -> jstring {
    let text = SESSION
        .get()
        .map(|s| on_air_text(&s.handle.model.load()))
        .unwrap_or_default();
    java_string(&mut env, &text)
}

#[allow(unsafe_code)]
// SAFETY: as above; `external fun notificationChannelName(): String`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_sanchezfauste_fausteplayer_Native_notificationChannelName(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
) -> jstring {
    let language = SESSION.get().and_then(|s| s.language.clone());
    let name = fp_app::i18n::I18n::new(language.as_deref()).tr("mobile-notification-channel");
    java_string(&mut env, &name)
}

#[allow(unsafe_code)]
// SAFETY: as above; `external fun setMediaFiles(Array<String>)`, one
// record per file (`MediaFile::from_record`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_sanchezfauste_fausteplayer_Native_setMediaFiles(
    mut env: JNIEnv<'_>,
    _this: JObject<'_>,
    paths: JObjectArray<'_>,
) {
    let count = env.get_array_length(&paths).unwrap_or(0);
    let mut files = Vec::with_capacity(usize::try_from(count).unwrap_or(0));
    for index in 0..count {
        // A failed call leaves a Java exception pending: no more JNI calls
        // until it is cleared, so the list stops there.
        let Ok(item) = env.get_object_array_element(&paths, index) else {
            let _ = env.exception_clear();
            break;
        };
        let item = JString::from(item);
        match env.get_string(&item) {
            Ok(record) => files.extend(fp_app::ui::media_browser::MediaFile::from_record(
                &String::from(record),
            )),
            Err(_) => {
                let _ = env.exception_clear();
                break;
            }
        }
        // Thousands of files: free each local reference as it goes.
        let _ = env.delete_local_ref(item);
    }
    // Tracks read before the permission was granted failed, and their files
    // did not change: the missing-file recheck would never look again.
    if MEDIA.set_files(files)
        && let Some(session) = SESSION.get()
    {
        let _ = session.requests.try_send(ServiceRequest::RecheckUnreadable);
    }
}

#[allow(unsafe_code)]
// SAFETY: as above; `external fun back()`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_sanchezfauste_fausteplayer_Native_back(
    _env: JNIEnv<'_>,
    _this: JObject<'_>,
) {
    BACK.store(true, Ordering::Release);
    if let Some(ctx) = REPAINT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_ref()
    {
        ctx.request_repaint();
    }
}

#[allow(unsafe_code)]
// SAFETY: as above; `external fun quit()`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_sanchezfauste_fausteplayer_Native_quit(
    _env: JNIEnv<'_>,
    _this: JObject<'_>,
) {
    // Waits for the services' final save: the shell ends the process next.
    if let Some(session) = SESSION.get() {
        session.stop_services();
        tracing::info!("saved; the app ends");
    }
}

/// Asks the activity to leave (`MainActivity.leave`): to the background,
/// or, with `close`, for good after the final save. Blocks for a JNI call:
/// run it off the interface thread.
pub(crate) fn leave(close: bool) -> Result<(), String> {
    call_activity("leave", "(Z)V", |_| Ok(JValueOwned::Bool(u8::from(close))))
}

/// Asks the activity to open `url` (`MainActivity.openUrl`, a view
/// intent). Blocks for a JNI call: run it off the interface thread.
pub(crate) fn open_url(url: &str) -> Result<(), String> {
    call_activity("openUrl", "(Ljava/lang/String;)V", |env| {
        env.new_string(url)
            .map(|text| JValueOwned::Object(text.into()))
    })
}

/// Calls `method` with signature `sig` on the running activity, with the
/// one argument `arg` makes.
fn call_activity(
    method: &str,
    sig: &str,
    arg: impl for<'a> FnOnce(&mut JNIEnv<'a>) -> jni::errors::Result<JValueOwned<'a>>,
) -> Result<(), String> {
    // Held through the call: the activity cannot end meanwhile.
    let current = ACTIVITY
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some((vm, activity)) = *current else {
        return Err("no activity".to_owned());
    };
    #[allow(unsafe_code)]
    // SAFETY: `vm` is the process's JavaVM pointer from android-activity
    // (`vm_as_ptr`), valid for the life of the process.
    let vm = unsafe { JavaVM::from_raw(vm as *mut jni::sys::JavaVM) }.map_err(|e| e.to_string())?;
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    #[allow(unsafe_code)]
    // SAFETY: `activity` is android-activity's global reference to the
    // running activity (`activity_as_ptr`), valid while `ACTIVITY` holds it
    // and we hold its lock. `JObject` never deletes it.
    let activity = unsafe { JObject::from_raw(activity as jni::sys::jobject) };
    let called =
        arg(&mut env).and_then(|value| env.call_method(&activity, method, sig, &[value.borrow()]));
    // Any failure may leave a Java exception pending; the thread must not
    // detach with one.
    if called.is_err() {
        let _ = env.exception_clear();
    }
    called.map(|_| ()).map_err(|e| e.to_string())
}
