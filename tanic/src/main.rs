use clap::Parser;
use miette::{IntoDiagnostic, Result};
use std::sync::{Arc, RwLock};

use crate::args::Args;
use tanic_core::config::ConnectionDetails;
use tanic_core::{TanicConfig, TanicMessage};
use tanic_svc::TanicSvc;
use tanic_tui::TanicTui;

mod args;
mod logging;

#[tokio::main]
async fn main() -> Result<()> {
    logging::init_tui_logger();

    let args = Args::try_parse().into_diagnostic()?;
    let config = TanicConfig::load().into_diagnostic()?;
    tracing::info!(?config, "loaded config");
    let config = Arc::new(RwLock::new(config));

    let (ui_tx, ui_rx) = tokio::sync::mpsc::channel(10);
    let (app_tx, app_rx) = tokio::sync::mpsc::channel(10);
    let svc_app_tx = ui_tx.clone();

    let ui_task = tokio::spawn(async move { TanicTui::start(app_rx, ui_tx).await });
    let svc_task = tokio::spawn(async move { TanicSvc::start(ui_rx, app_tx, config).await });

    if let Some(ref uri) = args.catalogue_uri {
        let connection = ConnectionDetails::new_anon(uri.clone());

        let message = TanicMessage::ConnectTo(connection);
        svc_app_tx.send(message).await.into_diagnostic()?;
    }

    tokio::select! {
        _ = ui_task => Ok(()),
        _ = svc_task => Ok(())
    }
}
