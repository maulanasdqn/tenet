use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::Handler;
use futures_util::StreamExt;
use tempfile::TempDir;
use tenet_errors::AppError;
use tokio::task::JoinHandle;

use crate::settings::RenderSettings;

const FALLBACK_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub struct Session {
    pub browser: Browser,
    pub reported_user_agent: String,
    alive: Arc<AtomicBool>,
    _handler: JoinHandle<()>,
    _user_data_dir: Option<TempDir>,
}

impl Session {
    pub async fn open(settings: &RenderSettings) -> Result<Self, AppError> {
        if settings.is_remote() {
            return Self::connect(settings).await;
        }
        Self::launch(settings).await
    }

    async fn connect(settings: &RenderSettings) -> Result<Self, AppError> {
        let (browser, handler) = Browser::connect(settings.chrome_ws_url.clone())
            .await
            .map_err(AppError::unavailable)?;
        tracing::info!(endpoint = %settings.chrome_ws_url, "connected to a remote chromium");
        Ok(Self::pump(browser, handler, None).await)
    }

    async fn launch(settings: &RenderSettings) -> Result<Self, AppError> {
        let user_data_dir = TempDir::new().map_err(AppError::internal)?;
        let config = build_config(settings, &user_data_dir)?;
        let (browser, handler) = Browser::launch(config)
            .await
            .map_err(AppError::unavailable)?;
        tracing::info!(
            chrome = settings.executable().as_deref().unwrap_or("bundled"),
            headful = settings.headful,
            "chromium launched"
        );
        Ok(Self::pump(browser, handler, Some(user_data_dir)).await)
    }

    async fn pump(browser: Browser, mut handler: Handler, user_data_dir: Option<TempDir>) -> Self {
        let alive = Arc::new(AtomicBool::new(true));
        let flag = Arc::clone(&alive);
        let task = tokio::spawn(async move {
            pump_handler(&mut handler).await;
            flag.store(false, Ordering::Relaxed);
            tracing::error!("chromium handler stream ended, browser is dead");
        });
        let reported_user_agent = reported_user_agent(&browser).await;
        Self {
            browser,
            reported_user_agent,
            alive,
            _handler: task,
            _user_data_dir: user_data_dir,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }
}

async fn reported_user_agent(browser: &Browser) -> String {
    match browser.version().await {
        Ok(version) if !version.user_agent.is_empty() => version.user_agent,
        _ => FALLBACK_USER_AGENT.to_owned(),
    }
}

async fn pump_handler(handler: &mut Handler) {
    while let Some(event) = handler.next().await {
        if event.is_err() {
            return;
        }
    }
}

fn build_config(
    settings: &RenderSettings,
    user_data_dir: &TempDir,
) -> Result<BrowserConfig, AppError> {
    let mut builder = BrowserConfig::builder()
        .user_data_dir(user_data_dir.path())
        .window_size(settings.viewport_width, settings.viewport_height);

    if settings.headful {
        builder = builder.with_head();
    }
    if settings.no_sandbox {
        builder = builder.no_sandbox().arg("disable-dev-shm-usage");
    }
    if let Some(path) = settings.executable() {
        builder = builder.chrome_executable(path.as_str());
    }
    for arg in tenet_stealth::STEALTH_ARGS {
        builder = builder.arg(*arg);
    }

    builder.build().map_err(AppError::internal)
}
