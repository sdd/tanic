use crate::iceberg_context::IcebergContext;
use http::Uri;
use iceberg::NamespaceIdent;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use tanic_core::config::ConnectionDetails;
use tanic_core::message::TanicMessage;
use tanic_core::Result;
use tanic_core::TanicConfig;
use tokio::sync::mpsc::{Receiver, Sender};

pub mod iceberg_context;

pub struct TanicSvc {
    should_exit: bool,
    rx: Receiver<TanicMessage>,
    tx: Sender<TanicMessage>,
    config: Arc<RwLock<TanicConfig>>,

    open_connections: Vec<IcebergContext>,
}

impl TanicSvc {
    pub async fn start(
        rx: Receiver<TanicMessage>,
        tx: Sender<TanicMessage>,
        config: Arc<RwLock<TanicConfig>>,
    ) -> Result<()> {
        let mut svc = TanicSvc::new(rx, tx, config);
        svc.event_loop().await
    }

    pub fn new(
        rx: Receiver<TanicMessage>,
        tx: Sender<TanicMessage>,
        config: Arc<RwLock<TanicConfig>>,
    ) -> Self {
        Self {
            config,
            open_connections: vec![],
            should_exit: false,
            rx,
            tx,
        }
    }

    async fn event_loop(&mut self) -> Result<()> {
        while !self.should_exit {
            let Some(message) = self.rx.recv().await else {
                break;
            };
            tracing::info!(?message, "svc received a message");

            match message {
                TanicMessage::Exit => {
                    self.should_exit = true;
                }

                TanicMessage::ConnectTo(connection_details) => {
                    self.connect_to(connection_details).await?;
                }

                _ => {}
            }
        }

        Ok(())
    }

    async fn connect_to(&mut self, conn_details: ConnectionDetails) -> Result<()> {
        let iceberg_context = IcebergContext::from_connection_details(&conn_details);
        let namespaces = iceberg_context.populate_namespaces().await?;

        self.open_connections = vec![iceberg_context];

        let namespaces = namespaces.read().unwrap().deref().clone();

        self.tx
            .send(TanicMessage::ShowNamespaces(namespaces))
            .await?;

        Ok(())
    }
}
