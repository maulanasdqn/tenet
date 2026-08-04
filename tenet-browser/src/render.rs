use std::sync::Arc;
use std::time::Duration;

use chromiumoxide::Page;
use tenet_errors::AppError;
use tenet_stealth::StealthProfile;
use tokio::sync::{Mutex, Semaphore};

use crate::capture::NetworkCapture;
use crate::launch::Session;
use crate::page::RenderedPage;
use crate::settings::RenderSettings;
use crate::stealth_apply;

const STORAGE_KEYS: &str = "Object.keys(window.localStorage || {})";

pub struct ChromiumRenderer {
    session: Mutex<Arc<Session>>,
    permits: Arc<Semaphore>,
    settings: RenderSettings,
}

impl ChromiumRenderer {
    pub async fn open(settings: RenderSettings, pool_size: usize) -> Result<Self, AppError> {
        let session = Arc::new(Session::open(&settings).await?);
        Ok(Self {
            session: Mutex::new(session),
            permits: Arc::new(Semaphore::new(pool_size.max(1))),
            settings,
        })
    }

    pub async fn render(&self, url: &str) -> Result<RenderedPage, AppError> {
        let _permit = Arc::clone(&self.permits)
            .acquire_owned()
            .await
            .map_err(AppError::internal)?;
        let session = self.session().await?;
        let page = session
            .browser
            .new_page("about:blank")
            .await
            .map_err(AppError::unavailable)?;

        self.disguise(&page, &session).await;
        let rendered = self.visit(&page, url).await;
        if let Err(err) = page.close().await {
            tracing::warn!(error = %err, "could not close the page");
        }
        rendered
    }

    async fn disguise(&self, page: &Page, session: &Session) {
        if !self.settings.stealth {
            return;
        }
        let profile =
            StealthProfile::from_user_agent(&session.reported_user_agent, self.settings.region());
        stealth_apply::apply(page, &profile).await;
    }

    async fn session(&self) -> Result<Arc<Session>, AppError> {
        let mut current = self.session.lock().await;
        if current.is_alive() {
            return Ok(Arc::clone(&current));
        }
        tracing::warn!("relaunching a dead chromium session");
        let fresh = Arc::new(Session::open(&self.settings).await?);
        *current = Arc::clone(&fresh);
        Ok(fresh)
    }

    async fn visit(&self, page: &Page, url: &str) -> Result<RenderedPage, AppError> {
        let capture = NetworkCapture::watch(page).await;
        self.navigate(page, url).await?;
        if self.settings.behavior {
            let plan = tenet_stealth::interaction_plan(
                self.settings.viewport_width,
                self.settings.viewport_height,
                url,
            );
            crate::humanize::humanize(page, &plan).await;
        }
        let settle = tenet_stealth::settle_delay(
            self.settings.settle_ms,
            self.settings.settle_jitter_ms,
            url,
        );
        tokio::time::sleep(Duration::from_millis(settle)).await;

        let html = page.content().await.map_err(AppError::unavailable)?;
        let final_url = page
            .url()
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| url.to_owned());
        let storage_keys = storage_keys(page).await;

        let Some(capture) = capture else {
            return Ok(RenderedPage {
                url: final_url,
                status: 200,
                headers: Vec::new(),
                html,
                captured: Vec::new(),
                storage_keys,
            });
        };
        let main = capture.main_response().await.unwrap_or_default();
        Ok(RenderedPage {
            url: final_url,
            status: if main.status == 0 { 200 } else { main.status },
            headers: main.headers,
            html,
            captured: capture.requests().await,
            storage_keys,
        })
    }

    async fn navigate(&self, page: &Page, url: &str) -> Result<(), AppError> {
        let deadline = Duration::from_secs(self.settings.nav_timeout_seconds);
        let navigation = async {
            page.goto(url).await.map_err(AppError::unavailable)?;
            page.wait_for_navigation()
                .await
                .map_err(AppError::unavailable)?;
            Ok::<(), AppError>(())
        };
        match tokio::time::timeout(deadline, navigation).await {
            Ok(result) => result,
            Err(_) => Err(AppError::Unavailable(format!(
                "{url} did not settle in time"
            ))),
        }
    }
}

async fn storage_keys(page: &Page) -> Vec<String> {
    let Ok(result) = page.evaluate(STORAGE_KEYS).await else {
        return Vec::new();
    };
    result.into_value::<Vec<String>>().unwrap_or_default()
}
