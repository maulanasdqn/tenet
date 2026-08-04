use std::future::pending;

use tokio::signal::unix::{signal, SignalKind};
use tokio_util::sync::CancellationToken;

pub fn shutdown_token() -> CancellationToken {
    let token = CancellationToken::new();
    let signalled = token.clone();
    tokio::spawn(async move {
        terminate_signal().await;
        tracing::info!("shutdown signal received, draining");
        signalled.cancel();
    });
    token
}

pub async fn terminate_signal() {
    tokio::select! {
        () = sigterm() => {}
        () = sigint() => {}
    }
}

async fn sigterm() {
    match signal(SignalKind::terminate()) {
        Ok(mut stream) => {
            stream.recv().await;
        }
        Err(err) => {
            tracing::error!(error = %err, "cannot listen for sigterm");
            pending::<()>().await;
        }
    }
}

async fn sigint() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!(error = %err, "cannot listen for sigint");
        pending::<()>().await;
    }
}
