//! One postcard file per analysed track. The key covers everything that can
//! change the result: the file (path, size, mtime), the analysis code
//! version, the analysis settings and the cover limits. File names start
//! with the analysis version, so entries of older versions, which can never
//! be read again, are swept by the analysis pool.

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::UNIX_EPOCH;

use fp_model::{AnalysisSettings, Limits};
use serde::{Deserialize, Serialize};

use crate::analyze::Analysis;

/// Bump when the analysis algorithm or output format changes.
pub const ANALYSIS_VERSION: u32 = 6;

#[derive(Serialize, Deserialize)]
struct CachedAnalysis {
    key: String,
    analysis: Analysis,
}

/// Identifies one file state plus the settings it was analysed with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheKey(String);

pub struct AnalysisCache {
    dir: PathBuf,
    max_bytes: u64,
    cover_limits: (u64, u32),
    tmp_counter: AtomicU64,
}

/// How the names of this analysis version's entries start.
fn version_prefix() -> String {
    format!("v{ANALYSIS_VERSION}-")
}

/// FNV-1a 64: a stable hash (unlike `DefaultHasher`) for cache file names.
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
    })
}

impl AnalysisCache {
    /// Opens (or creates on first store) a cache in `dir`, without touching
    /// the disk: `sweep` tidies it, off the caller's thread.
    pub fn new(dir: PathBuf, limits: &Limits) -> Self {
        Self {
            dir,
            max_bytes: limits.max_state_file_bytes,
            cover_limits: (limits.max_cover_bytes, limits.max_cover_pixels),
            tmp_counter: AtomicU64::new(0),
        }
    }

    /// Removes temporary files left by an interrupted run and entries of
    /// other analysis versions, which can never match a key again. The
    /// analysis pool runs it before its first job.
    pub fn sweep(&self) {
        let current = version_prefix();
        let Ok(entries) = fs::read_dir(&self.dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let stale = match path.extension().and_then(|e| e.to_str()) {
                Some("tmp") => true,
                Some("bin") => !entry.file_name().to_string_lossy().starts_with(&current),
                _ => false,
            };
            if stale {
                let _ = fs::remove_file(path);
            }
        }
    }

    /// The key of `path` as it is right now, or `None` if it cannot be read.
    pub fn key(&self, path: &Path, settings: &AnalysisSettings) -> Option<CacheKey> {
        let canonical = path.canonicalize().ok()?;
        let meta = fs::metadata(&canonical).ok()?;
        let mtime = meta
            .modified()
            .ok()?
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos();
        let (cover_bytes, cover_pixels) = self.cover_limits;
        Some(CacheKey(format!(
            "{}|{}|{mtime}|{ANALYSIS_VERSION}|{settings:?}|{cover_bytes}|{cover_pixels}",
            canonical.display(),
            meta.len()
        )))
    }

    fn file_for(&self, key: &CacheKey) -> PathBuf {
        self.dir.join(format!(
            "{}{:016x}.bin",
            version_prefix(),
            fnv1a(key.0.as_bytes())
        ))
    }

    /// The cached analysis for `key`. A corrupt or oversized entry is
    /// removed so it is recomputed.
    pub fn load_key(&self, key: &CacheKey) -> Option<Analysis> {
        let file = self.file_for(key);
        if fs::metadata(&file).ok()?.len() > self.max_bytes {
            let _ = fs::remove_file(&file);
            return None;
        }
        let bytes = fs::read(&file).ok()?;
        match postcard::from_bytes::<CachedAnalysis>(&bytes) {
            Ok(cached) if cached.key == key.0 => Some(cached.analysis),
            Ok(_) => None, // a hash collision: another file's entry
            Err(_) => {
                let _ = fs::remove_file(&file);
                None
            }
        }
    }

    /// Stores `analysis` under `key` (written to a unique temporary file,
    /// then renamed into place).
    pub fn store_key(&self, key: &CacheKey, analysis: &Analysis) -> io::Result<()> {
        let bytes = postcard::to_allocvec(&CachedAnalysis {
            key: key.0.clone(),
            analysis: analysis.clone(),
        })
        .map_err(io::Error::other)?;
        if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > self.max_bytes {
            return Err(io::Error::other(
                "analysis larger than the state file limit",
            ));
        }
        fs::create_dir_all(&self.dir)?;
        let file = self.file_for(key);
        let n = self.tmp_counter.fetch_add(1, Ordering::Relaxed);
        let tmp = file.with_extension(format!("{}-{n}.tmp", std::process::id()));
        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&bytes)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, &file).inspect_err(|_| {
            let _ = fs::remove_file(&tmp);
        })
    }

    pub fn load(&self, path: &Path, settings: &AnalysisSettings) -> Option<Analysis> {
        self.load_key(&self.key(path, settings)?)
    }

    pub fn store(
        &self,
        path: &Path,
        settings: &AnalysisSettings,
        analysis: &Analysis,
    ) -> io::Result<()> {
        let key = self
            .key(path, settings)
            .ok_or_else(|| io::Error::other("file metadata unavailable"))?;
        self.store_key(&key, analysis)
    }
}
