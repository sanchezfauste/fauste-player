//! The calls of the Kotlin class `com.sanchezfauste.fausteplayer.Native`.
//! Each entry point only reads or stores a value; none blocks.

use std::sync::atomic::Ordering;

use jni::JNIEnv;
use jni::objects::{JObject, JObjectArray, JString};
use jni::sys::{jboolean, jint, jstring};

use crate::android::{INSETS, SESSION};
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
// SAFETY: as above; `external fun setMediaFiles(Array<String>)`.
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
            Ok(path) => files.push(std::path::PathBuf::from(String::from(path))),
            Err(_) => {
                let _ = env.exception_clear();
                break;
            }
        }
        // Thousands of files: free each local reference as it goes.
        let _ = env.delete_local_ref(item);
    }
    MEDIA.set_files(files);
}
