use std::time::Duration;

use chromiumoxide::cdp::browser_protocol::input::{
    DispatchMouseEventParams, DispatchMouseEventType,
};
use chromiumoxide::layout::Point;
use chromiumoxide::Page;
use tenet_stealth::Gesture;

const MOVE_INTERVAL_MS: u64 = 14;

pub async fn humanize(page: &Page, plan: &[Gesture]) {
    let mut cursor = (0.0, 0.0);
    for gesture in plan {
        match gesture {
            Gesture::Move { x, y } => {
                if let Err(err) = page.move_mouse(Point { x: *x, y: *y }).await {
                    tracing::debug!(error = %err, "mouse move failed");
                }
                cursor = (*x, *y);
                tokio::time::sleep(Duration::from_millis(MOVE_INTERVAL_MS)).await;
            }
            Gesture::Scroll { delta_y } => scroll(page, cursor, *delta_y).await,
            Gesture::Dwell { ms } => tokio::time::sleep(Duration::from_millis(*ms)).await,
        }
    }
}

async fn scroll(page: &Page, cursor: (f64, f64), delta_y: f64) {
    let mut params =
        DispatchMouseEventParams::new(DispatchMouseEventType::MouseWheel, cursor.0, cursor.1);
    params.delta_x = Some(0.0);
    params.delta_y = Some(delta_y);
    if let Err(err) = page.execute(params).await {
        tracing::debug!(error = %err, "scroll failed");
    }
}
