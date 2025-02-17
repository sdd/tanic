//! Iceberg Context

use std::sync::Arc;
use std::sync::RwLock;

use futures::stream::StreamExt;
use iceberg::{Catalog, NamespaceIdent, TableIdent};
use iceberg_catalog_rest::{RestCatalog, RestCatalogConfig};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::watch::Receiver;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::WatchStream;

use tanic_core::config::ConnectionDetails;
use tanic_core::message::{NamespaceDeets, TableDeets};
use tanic_core::{Result, TanicError};
use tokio::sync::mpsc::{channel, Receiver as MpscReceiver, Sender as MpscSender};
use tokio_stream::wrappers::ReceiverStream;

use crate::state::{TanicAction, TanicAppState, TanicIcebergState};

type ActionTx = UnboundedSender<TanicAction>;
type IceCtxRef = Arc<RwLock<IcebergContext>>;

const JOB_STREAM_CONCURRENCY: usize = 1;

#[derive(Debug, Default)]
struct IcebergContext {
    connection_details: Option<ConnectionDetails>,

    /// Iceberg Catalog
    catalog: Option<Arc<dyn Catalog>>,

    namespaces: Vec<NamespaceDeets>,
    tables: Vec<TableDeets>,

    #[allow(unused)] // TODO: cancellation
    pub cancellable_action: Option<JoinHandle<()>>,
}

/// Iceberg Context
#[derive(Debug)]
pub struct IcebergContextManager {
    action_tx: ActionTx,
    iceberg_context: IceCtxRef,
    state_ref: Arc<RwLock<TanicAppState>>,
}

#[derive(Debug)]
enum IcebergTask {
    Namespaces,
    TablesForNamespace(NamespaceDeets),
    SummaryForTable(TableDeets),
}

impl IcebergContextManager {
    pub fn new(action_tx: ActionTx, state_ref: Arc<RwLock<TanicAppState>>) -> Self {
        Self {
            action_tx,
            state_ref,
            iceberg_context: Arc::new(RwLock::new(IcebergContext::default())),
        }
    }

    pub async fn event_loop(&self, state_rx: Receiver<()>) -> Result<()> {
        let mut state_stream = WatchStream::new(state_rx);
        let (job_queue_tx, job_queue_rx) = channel(10);

        let mut current_conn_details: Option<ConnectionDetails> = None;

        tokio::spawn({
            let action_tx = self.action_tx.clone();
            let job_queue_tx = job_queue_tx.clone();
            let iceberg_ctx = self.iceberg_context.clone();
            async move {
                tracing::debug!("await job_handler()");
                Self::job_handler(job_queue_rx, job_queue_tx, action_tx, iceberg_ctx).await
            }
        });

        tracing::debug!("await state_stream.next() 1");
        let mut next_item = state_stream.next().await;
        tracing::debug!("await state_stream.next() 1 complete");
        while next_item.is_some() {
            let new_conn_details = {
                tracing::debug!("self.state_ref.read()");
                let state = self.state_ref.read().unwrap();

                match &state.iceberg {
                    TanicIcebergState::ConnectingTo(ref new_conn_details) => {
                        Some(new_conn_details.clone())
                    }
                    TanicIcebergState::Exiting => {
                        break;
                    }
                    _ => None,
                }
            };
            tracing::debug!("self.state_ref.read() done");

            if let Some(new_conn_details) = new_conn_details {
                if Some(new_conn_details.clone()) != current_conn_details {
                    current_conn_details = Some(new_conn_details.clone());

                    tracing::debug!("await self.connect_to()");
                    self.connect_to(&new_conn_details, job_queue_tx.clone())
                        .await?;
                    tracing::debug!("await self.connect_to() done");

                    // begin crawl
                    tracing::debug!("await job_queue_tx.send()");
                    let _ = job_queue_tx.send(IcebergTask::Namespaces).await;
                    tracing::debug!("await job_queue_tx.send() done");
                }
            }

            tracing::debug!("await state_stream.next() 2");
            next_item = state_stream.next().await;
            tracing::debug!("await state_stream.next() 2 complete");
        }

        Ok(())
    }

    async fn connect_to(
        &self,
        new_conn_details: &ConnectionDetails,
        _job_queue_tx: MpscSender<IcebergTask>,
    ) -> Result<()> {
        {
            tracing::debug!("self.iceberg_context.read()");
            let ctx = self.iceberg_context.read().unwrap();
            if let Some(ref existing_conn_details) = ctx.connection_details {
                if new_conn_details == existing_conn_details {
                    // do nothing, already connected to this catalog
                    return Ok(());
                }
            }
        }
        tracing::debug!("self.iceberg_context.read() done");

        {
            tracing::debug!("self.iceberg_context.write()");
            let mut ctx = self.iceberg_context.write().unwrap();
            ctx.connect_to(new_conn_details);
        }
        tracing::debug!("self.iceberg_context.write() done");

        Ok(())
    }

    async fn populate_namespaces(
        ctx: IceCtxRef,
        action_tx: ActionTx,
        job_queue_tx: MpscSender<IcebergTask>,
    ) -> Result<()> {
        let root_namespaces = {
            let catalog = {
                tracing::debug!("ctx.read()");
                let r_ctx = ctx.read().unwrap();

                let Some(ref catalog) = r_ctx.catalog else {
                    return Err(TanicError::unexpected(
                        "Attempted to populate namespaces when catalog not initialised",
                    ));
                };

                catalog.clone()
            };
            tracing::debug!("ctx.read() done");

            tracing::debug!("catalog.list_namespaces(None).await");
            let res = catalog.list_namespaces(None).await?;
            tracing::debug!("catalog.list_namespaces(None).await done");

            res
        };

        let namespaces = root_namespaces
            .into_iter()
            .map(|ns| NamespaceDeets::from_parts(ns.inner()))
            .collect::<Vec<_>>();

        {
            let namespaces = namespaces.clone();
            tracing::debug!("ctx.write()");
            ctx.write().unwrap().namespaces = namespaces;
        }
        tracing::debug!("ctx.write() done");

        action_tx
            .send(TanicAction::UpdateNamespacesList(
                namespaces
                    .iter()
                    .map(|ns| ns.name.clone())
                    .collect::<Vec<_>>(),
            ))
            .map_err(|err| TanicError::UnexpectedError(err.to_string()))?;

        for namespace in namespaces {
            tracing::debug!("job_queue_tx.send await");
            let _ = job_queue_tx
                .send(IcebergTask::TablesForNamespace(namespace.clone()))
                .await;
            tracing::debug!("job_queue_tx.send await done");
        }

        Ok(())
    }

    async fn populate_tables(
        ctx: IceCtxRef,
        action_tx: ActionTx,
        namespace: NamespaceDeets,
        job_queue_tx: MpscSender<IcebergTask>,
    ) -> Result<()> {
        let namespace_ident = NamespaceIdent::from_strs(namespace.parts.clone())?;
        let tables = {
            let catalog = {
                tracing::debug!("ctx.read()");
                let r_ctx = ctx.read().unwrap();

                let Some(ref catalog) = r_ctx.catalog else {
                    return Err(TanicError::unexpected(
                        "Attempted to populate namespaces when catalog not initialised",
                    ));
                };

                catalog.clone()
            };
            tracing::debug!("ctx.read() done");

            tracing::debug!("catalog.list_tables(&namespace_ident).await");
            let res = catalog.list_tables(&namespace_ident).await?;
            tracing::debug!("catalog.list_tables(&namespace_ident).await done");

            res
        };

        let tables = tables
            .into_iter()
            .map(|ti| TableDeets {
                namespace: namespace.parts.clone(),
                name: ti.name().to_string(),
                row_count: 1,
            })
            .collect::<Vec<_>>();

        {
            let tables = tables.clone();
            tracing::debug!("ctx.write()");
            ctx.write().unwrap().tables = tables;
        }
        tracing::debug!("ctx.write() done");

        action_tx
            .send(TanicAction::UpdateNamespaceTableList(
                namespace.name.clone(),
                tables.iter().map(|t| &t.name).cloned().collect(),
            ))
            .map_err(TanicError::unexpected)?;

        for table in tables {
            tracing::debug!("job_queue_tx.send await");
            tracing::info!(?table, "sending SummaryForTable");
            let _ = job_queue_tx
                .send(IcebergTask::SummaryForTable(table.clone()))
                .await;
            tracing::debug!("job_queue_tx.send await done");
        }

        Ok(())
    }

    async fn populate_table_summary(
        ctx: IceCtxRef,
        action_tx: ActionTx,
        table: TableDeets,
        _job_queue_tx: MpscSender<IcebergTask>,
    ) -> Result<()> {
        let namespace_ident = NamespaceIdent::from_strs(table.namespace.clone())?;
        let table_ident = TableIdent::new(namespace_ident.clone(), table.name.clone());

        let loaded_table = {
            let catalog = {
                tracing::debug!("ctx.read()");
                let r_ctx = ctx.read().unwrap();

                let Some(ref catalog) = r_ctx.catalog else {
                    return Err(TanicError::unexpected(
                        "Attempted to populate table summary when catalog not initialised",
                    ));
                };

                catalog.clone()
            };
            tracing::debug!("ctx.read() done");

            tracing::debug!("catalog.load_table(&table_ident).await");
            let res = catalog.load_table(&table_ident).await?;
            tracing::debug!("catalog.load_table(&table_ident).await done");

            res
        };

        let summary = loaded_table
            .metadata()
            .current_snapshot()
            .unwrap()
            .summary();
        // tracing::info!(?summary);

        action_tx
            .send(TanicAction::UpdateTableSummary {
                namespace: namespace_ident.to_url_string(),
                table_name: table_ident.name.clone(),
                table_summary: summary.additional_properties.clone(),
            })
            .map_err(TanicError::unexpected)?;

        Ok(())
    }
}

impl IcebergContext {
    /// Create a new Iceberg Context from a Uri
    pub fn connect_to(&mut self, connection_details: &ConnectionDetails) {
        self.connection_details = Some(connection_details.clone());

        let mut uri_str = connection_details.uri.to_string();
        uri_str.pop();

        let config = RestCatalogConfig::builder().uri(uri_str).build();
        self.catalog = Some(Arc::new(RestCatalog::new(config)));

        self.namespaces = vec![];
        self.tables = vec![];
    }
}

impl IcebergContextManager {
    async fn job_handler(
        job_queue_rx: MpscReceiver<IcebergTask>,
        job_queue_tx: MpscSender<IcebergTask>,
        action_tx: ActionTx,
        iceberg_ctx: IceCtxRef,
    ) {
        let job_stream = ReceiverStream::new(job_queue_rx);

        job_stream
            .map(|task| {
                (
                    task,
                    iceberg_ctx.clone(),
                    action_tx.clone(),
                    job_queue_tx.clone(),
                )
            })
            .for_each_concurrent(
                JOB_STREAM_CONCURRENCY,
                async move |(task, iceberg_ctx, action_tx, job_queue_tx)| {
                    match task {
                        IcebergTask::Namespaces => {
                            tracing::debug!("populate_namespaces.await");
                            let _ = IcebergContextManager::populate_namespaces(
                                iceberg_ctx,
                                action_tx,
                                job_queue_tx,
                            )
                            .await;
                            tracing::debug!("populate_namespaces.await done");
                        }

                        IcebergTask::TablesForNamespace(namespace) => {
                            tracing::debug!("populate_tables.await");
                            let _ = IcebergContextManager::populate_tables(
                                iceberg_ctx,
                                action_tx,
                                namespace,
                                job_queue_tx,
                            )
                            .await;
                            tracing::debug!("populate_tables.await done");
                        }

                        IcebergTask::SummaryForTable(table) => {
                            tracing::debug!("populate_table_summary.await");
                            let _ = IcebergContextManager::populate_table_summary(
                                iceberg_ctx,
                                action_tx,
                                table,
                                job_queue_tx,
                            )
                            .await;
                            tracing::debug!("populate_table_summary.await done");
                        } // _ => {}
                    }
                },
            )
            .await;
        // }).await;
    }
}
