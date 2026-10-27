//! The Android platform: pickers through the media browser (MB7), read-only
//! music (spec §2) and the audio focus the Kotlin shell reports.

use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use fp_app::platform::Platform;
use fp_app::ui::media_browser::{BrowseMode, MediaBrowser};

/// Set by `Native.setAudioFocusLost` (MB4).
pub(crate) static AUDIO_FOCUS_LOST: AtomicBool = AtomicBool::new(false);

/// The device's music, fed by `Native.setMediaFiles`, and the pickers'
/// requests. Process-wide: a request outlives the activity that drew it,
/// and the next activity's interface shows it.
pub(crate) static MEDIA: LazyLock<MediaBrowser> = LazyLock::new(MediaBrowser::default);

pub struct AndroidPlatform;

impl Platform for AndroidPlatform {
    fn pick_audio_files(&self, _filter_name: &str, start: Option<PathBuf>) -> Vec<PathBuf> {
        MEDIA.browse(BrowseMode::AudioFiles, start)
    }

    fn pick_file(
        &self,
        _filter_name: &str,
        extensions: &[&str],
        start: Option<PathBuf>,
    ) -> Option<PathBuf> {
        let extensions = extensions.iter().map(|e| (*e).to_owned()).collect();
        MEDIA
            .browse(BrowseMode::OneFile { extensions }, start)
            .into_iter()
            .next()
    }

    fn pick_folder(&self, start: Option<PathBuf>) -> Option<PathBuf> {
        MEDIA.browse(BrowseMode::Folder, start).into_iter().next()
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
