//! One postcard file per analysed track. The key covers everything that can
//! change the result: the file (path, size, mtime), the analysis code
//! version and the analysis settings.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use fp_model::AnalysisSettings;
use serde::{Deserialize, Serialize};

use crate::analyze::Analysis;

/// Bump when the analysis algorithm or output format changes.
pub const ANALYSIS_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct CachedAnalysis {
    key: String,
    analysis: Analysis,
}

pub struct AnalysisCache {
    dir: PathBuf,
    max_bytes: u64,
}

/// FNV-1a 64: a stable hash (unlike `DefaultHasher`) for cache file names.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

impl AnalysisCache {
    pub fn new(dir: PathBuf, max_bytes: u64) -> Self {
        Self { dir, max_bytes }
    }

    fn key(path: &Path, settings: &AnalysisSettings) -> Option<String> {
        let canonical = path.canonicalize().ok()?;
        let meta = fs::metadata(&canonical).ok()?;
        let mtime = meta
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos();
        Some(format!(
            "{}|{}|{mtime}|{ANALYSIS_VERSION}|{settings:?}",
            canonical.display(),
            meta.len()
        ))
    }

    fn file_for(&self, key: &str) -> PathBuf {
        self.dir.join(format!("{:016x}.bin", fnv1a(key.as_bytes())))
    }

    /// The cached analysis if the file and settings are unchanged. A corrupt
    /// entry is removed so it is recomputed.
    pub fn load(&self, path: &Path, settings: &AnalysisSettings) -> Option<Analysis> {
        let key = Self::key(path, settings)?;
        let file = self.file_for(&key);
        if fs::metadata(&file).ok()?.len() > self.max_bytes {
            let _ = fs::remove_file(&file);
            return None;
        }
        let bytes = fs::read(&file).ok()?;
        match postcard::from_bytes::<CachedAnalysis>(&bytes) {
            Ok(cached) if cached.key == key => Some(cached.analysis),
            Ok(_) => None, // a hash collision: another file's entry
            Err(_) => {
                let _ = fs::remove_file(&file);
                None
            }
        }
    }

    pub fn store(
        &self,
        path: &Path,
        settings: &AnalysisSettings,
        analysis: &Analysis,
    ) -> io::Result<()> {
        let key = Self::key(path, settings)
            .ok_or_else(|| io::Error::other("file metadata unavailable"))?;
        let bytes = postcard::to_allocvec(&CachedAnalysis {
            key: key.clone(),
            analysis: analysis.clone(),
        })
        .map_err(io::Error::other)?;
        fs::create_dir_all(&self.dir)?;
        let file = self.file_for(&key);
        let tmp = file.with_extension("tmp");
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&bytes)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &file)
    }
}
