//! The running session: saved state, audio engine, conductor, analysis and
//! services. The desktop binary and the mobile shell start it the same way
//! (mobile spec §4); each adds its own window around it.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Instant;

use crossbeam_channel::Sender;
use fp_analysis::analyzer::Analyzer;
use fp_analysis::cache::AnalysisCache;
use fp_backends::{AudioBackend, Availability, NullBackend};
use fp_engine::conductor::{Conductor, ConductorHandle};
use fp_engine::engine::{Engine, EngineSettings};
use fp_engine::worker::file_opener;
use fp_store::{AppPaths, Store};

use crate::activity::ActivityCell;
use crate::crash::ReportCap;
use crate::i18n::I18n;
use crate::services::{MediaCache, ServiceRequest, Services, ServicesHandle};
use crate::ui::app::AppUi;

/// Background analysis threads: enough to keep up with imports without
/// competing with the audio threads.
const ANALYSIS_THREADS: usize = 2;

pub struct AppSession {
    pub handle: Arc<ConductorHandle>,
    pub requests: Sender<ServiceRequest>,
    pub activity: ActivityCell,
    pub faults: Arc<AtomicU64>,
    pub media: MediaCache,
    /// The audio systems in use, Null last.
    pub backends: Vec<Arc<dyn AudioBackend>>,
    /// The configured interface language (`None` follows the OS).
    pub language: Option<String>,
    services: ServicesHandle,
    analysis_dir: PathBuf,
}

impl AppSession {
    /// Loads the saved state and starts the engine on `backends` (Null is
    /// added as the last resort), the conductor, the analysis and the
    /// services thread. Nothing plays (rule 10).
    pub fn start(
        paths: AppPaths,
        backends: Vec<Arc<dyn AudioBackend>>,
        report_cap: Option<ReportCap>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let store = Store::new(paths.clone(), fp_model::Limits::default());
        let default_name = I18n::new(None).tr("default-playlist-name");
        let loaded = store.load(&default_name);
        for warning in &loaded.warnings {
            tracing::warn!(%warning, "while loading the saved state");
        }
        let config = loaded.state.config.clone();
        if let Some(cap) = &report_cap {
            cap.set(config.limits.max_crash_reports);
        }

        let mut backends = backends;
        backends.push(Arc::new(NullBackend) as Arc<dyn AudioBackend>);
        let availability: Vec<(String, bool)> = backends
            .iter()
            .map(|b| (b.id().0, b.availability() == Availability::Available))
            .collect();
        let settings = EngineSettings::from_config(&config);
        let engine = Engine::new(backends.clone(), settings, file_opener());
        let in_use = engine.backend_in_use().to_owned();
        tracing::info!(backend = %in_use, ?availability, "audio systems");
        let (conductor, handle) =
            Conductor::new(loaded.state, loaded.actions, engine, Instant::now());
        let handle = Arc::new(conductor.spawn(handle)?);

        let analysis_dir = paths.cache_dir.join("analysis");
        let cache = AnalysisCache::new(analysis_dir.clone());
        let analyzer = Analyzer::spawn(
            ANALYSIS_THREADS,
            config.analysis.clone(),
            config.limits.clone(),
            Some(cache),
        )?;
        let media = MediaCache::default();
        let mut services = Services::new(handle.clone(), store, analyzer, media.clone());
        if let Some(cap) = report_cap {
            services = services.with_report_cap(cap);
        }
        let requests = services.requests();
        let faults = services.faults();
        let activity = services.activity();
        let services = services.spawn()?;
        Ok(Self {
            handle,
            requests,
            activity,
            faults,
            media,
            backends,
            language: config.ui.language,
            services,
            analysis_dir,
        })
    }

    /// A new interface on this session; a mobile shell builds one per
    /// activity.
    pub fn app_ui(&self) -> AppUi {
        AppUi::new(
            self.handle.clone(),
            I18n::new(self.language.as_deref()),
            self.media.clone(),
        )
        .with_services(self.requests.clone())
        .with_analysis_activity(self.activity.clone())
        .with_service_faults(self.faults.clone())
        .with_backends(self.backends.clone())
    }

    /// Where the analysis cache lives.
    pub fn analysis_dir(&self) -> PathBuf {
        self.analysis_dir.clone()
    }

    /// Stops the services after their final save, then the engine. Every
    /// other part that can send commands must have stopped already.
    pub fn shutdown(self) {
        self.services.shutdown();
        stop_engine(self.handle);
    }
}

/// Stops the conductor and the engine, closing the output streams, now.
/// Every other holder of the handle must have stopped already.
fn stop_engine(handle: Arc<ConductorHandle>) {
    match Arc::try_unwrap(handle) {
        Ok(conductor) => {
            drop(conductor);
            tracing::info!("the audio engine is stopped");
        }
        Err(handle) => tracing::warn!(
            holders = Arc::strong_count(&handle) - 1,
            "the audio engine is still held by another part of the application; \
             it stops when the process ends"
        ),
    }
}
