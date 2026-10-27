//! The Android platform: interim pickers until the media browser (MB7),
//! read-only music (spec §2) and the audio focus the Kotlin shell reports.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use fp_app::platform::Platform;

/// Set by `Native.setAudioFocusLost` (MB4).
pub(crate) static AUDIO_FOCUS_LOST: AtomicBool = AtomicBool::new(false);

/// Where the interim pickers look.
const MUSIC_DIR: &str = "/sdcard/Music";

pub struct AndroidPlatform;

impl Platform for AndroidPlatform {
    /// Every audio file directly in the music folder, sorted (interim until
    /// the media browser, MB7).
    fn pick_audio_files(&self, _filter_name: &str, _start: Option<PathBuf>) -> Vec<PathBuf> {
        fp_app::ui::files::audio_paths(&[PathBuf::from(MUSIC_DIR)])
    }

    fn pick_file(
        &self,
        _filter_name: &str,
        _extensions: &[&str],
        _start: Option<PathBuf>,
    ) -> Option<PathBuf> {
        None
    }

    fn pick_folder(&self, _start: Option<PathBuf>) -> Option<PathBuf> {
        Some(PathBuf::from(MUSIC_DIR))
    }

    fn save_file(
        &self,
        _filter_name: &str,
        _extensions: &[&str],
        _file_name: String,
    ) -> Option<PathBuf> {
        None
    }

    fn open_url(&self, _url: &str) -> Result<(), String> {
        Err("not available yet".to_owned())
    }

    fn audio_focus_lost(&self) -> bool {
        AUDIO_FOCUS_LOST.load(Ordering::Relaxed)
    }

    fn can_write_user_files(&self) -> bool {
        false
    }
}
