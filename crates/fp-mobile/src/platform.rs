//! The Android platform: pickers through the media browser (MB7), links
//! through the activity, read-only music (spec §2) and the audio focus the
//! Kotlin shell reports.

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
        // Images, playlists and cart pages have no picker yet.
        let mode = BrowseMode::one_file(extensions)?;
        MEDIA.browse(mode, start).into_iter().next()
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

    /// Through the activity, on a helper thread (a JNI call; rule 8).
    fn open_url(&self, url: &str) -> Result<(), String> {
        let url = url.to_owned();
        std::thread::Builder::new()
            .name("fp-open".to_owned())
            .spawn(move || {
                if let Err(error) = crate::jni::open_url(&url) {
                    tracing::warn!(%error, %url, "could not open the link");
                }
            })
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn audio_focus_lost(&self) -> bool {
        AUDIO_FOCUS_LOST.load(Ordering::Relaxed)
    }

    fn can_write_user_files(&self) -> bool {
        false
    }
}
