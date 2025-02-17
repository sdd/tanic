use iceberg::spec::{DataFile, Manifest, ManifestList, Snapshot};
use iceberg::table::Table;
use indexmap::IndexMap;
use parquet::file::metadata::ParquetMetaData;
use std::collections::HashMap;
use tanic_core::config::ConnectionDetails;

const TABLE_SUMMARY_KEY_ROW_COUNT: &str = "total-records";

#[derive(Debug)]
pub enum TanicAction {
    Exit,

    ConnectTo(ConnectionDetails),

    // Iceberg metadata update actions
    UpdateNamespacesList(Vec<String>),
    UpdateNamespaceProperties(String, HashMap<String, String>),
    UpdateNamespaceTableList(String, Vec<String>),
    UpdateTable {
        namespace: String,
        table_name: String,
        table: Table,
    },
    UpdateTableSummary {
        namespace: String,
        table_name: String,
        table_summary: HashMap<String, String>,
    },
    UpdateTableCurrentSnapshot {
        namespace: String,
        table_name: String,
        snapshot: Snapshot,
    },
    UpdateTableCurrentManifestList {
        namespace: String,
        table_name: String,
        manifest_list: Box<ManifestList>,
    },
    UpdateTableManifest {
        namespace: String,
        table_name: String,
        manifest: Box<Manifest>,
        file_path: String,
    },
    UpdateTableDataFile {
        namespace: String,
        table_name: String,
        data_file: Box<DataFile>,
    },
    UpdateTableParquetMetaData {
        namespace: String,
        table_name: String,
        file_path: String,
        metadata: Box<ParquetMetaData>,
    },

    ///// UI Actions ///////
    FocusPrevNamespace,
    FocusNextNamespace,
    SelectNamespace,

    FocusPrevTable,
    FocusNextTable,
    SelectTable,

    Escape,

    FocusNextPartition,
    FocusPrevPartition,
    SelectPartition,

    FocusNextDataFile,
    FocusPrevDataFile,
    SelectDataFile,
}

#[derive(Clone, Debug, Default)]
pub struct TanicAppState {
    pub iceberg: TanicIcebergState,
    pub ui: TanicUiState,
}

#[derive(Clone, Debug, Default)]
pub enum TanicIcebergState {
    #[default]
    Initializing,
    ConnectingTo(ConnectionDetails),
    Connected(RetrievedIcebergMetadata),
    Exiting,
}

#[derive(Clone, Debug)]
pub struct RetrievedIcebergMetadata {
    pub namespaces: IndexMap<String, NamespaceDescriptor>,
}

#[derive(Clone, Debug)]
pub struct NamespaceDescriptor {
    pub name: String,
    properties: Option<HashMap<String, String>>,
    pub tables: Option<IndexMap<String, TableDescriptor>>,

    pub row_count: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct TableDescriptor {
    pub name: String,
    #[allow(unused)]
    namespace: Vec<String>,
    current_snapshot_summary: Option<HashMap<String, String>>,
    table: Option<Table>,
    current_snapshot: Option<Snapshot>,
    current_manifest_list: Option<ManifestList>,
    manifests: IndexMap<String, Manifest>,
    datafiles: HashMap<String, DataFile>,
    parquet_metadata: HashMap<String, ParquetMetaData>,

    row_count: Option<u64>,
}

impl TableDescriptor {
    pub fn row_count(&self) -> Option<u64> {
        self.row_count
    }
}

#[derive(Clone, Debug, Default)]
pub enum TanicUiState {
    #[default]
    SplashScreen,
    ViewingNamespacesList(ViewingNamespacesListState),
    ViewingTablesList(ViewingTablesListState),
    Exiting,
}

#[derive(Clone, Debug)]
pub struct ViewingNamespacesListState {
    pub selected_idx: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct ViewingTablesListState {
    pub namespaces: ViewingNamespacesListState,
    pub selected_idx: Option<usize>,
}

impl TanicAppState {
    pub(crate) fn update(mut self, action: TanicAction) -> Self {
        match (action, &mut self) {
            (TanicAction::Exit, _) => {
                self.iceberg = TanicIcebergState::Exiting;
                self.ui = TanicUiState::Exiting;
            }

            (TanicAction::ConnectTo(conn_details), _) => {
                self.iceberg = TanicIcebergState::ConnectingTo(conn_details);
                self.ui = TanicUiState::SplashScreen;
            }

            (TanicAction::UpdateNamespacesList(namespaces), _) => {
                let selected_idx = if namespaces.is_empty() { None } else { Some(0) };

                let namespaces = IndexMap::from_iter(namespaces.iter().map(|ns| {
                    (
                        ns.clone(),
                        NamespaceDescriptor {
                            name: ns.clone(),
                            properties: None,
                            tables: None,
                            row_count: None,
                        },
                    )
                }));

                self.iceberg =
                    TanicIcebergState::Connected(RetrievedIcebergMetadata { namespaces });
                self.ui = TanicUiState::ViewingNamespacesList(ViewingNamespacesListState {
                    selected_idx,
                });
            }

            (TanicAction::UpdateNamespaceProperties(namespace, properties), prev_state) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    tracing::error!("panic! not connected");
                    panic!();
                };

                let Some(namespacce_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    tracing::error!("panic! namespace not found");
                    panic!();
                };

                namespacce_desc.properties = Some(properties);
            }

            (TanicAction::UpdateNamespaceTableList(namespace, table_names), prev_state) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    tracing::error!("panic! not connected");
                    panic!();
                };

                let Some(namespacce_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    tracing::error!("panic! namepsace  not found");
                    panic!();
                };

                namespacce_desc.tables =
                    Some(IndexMap::from_iter(table_names.into_iter().map(|name| {
                        (
                            name.clone(),
                            TableDescriptor {
                                name,
                                namespace: namespace
                                    .split(" ")
                                    .map(|s| s.to_string())
                                    .collect::<Vec<_>>(),
                                current_snapshot_summary: None,
                                table: None,
                                current_snapshot: None,
                                current_manifest_list: None,
                                manifests: IndexMap::default(),
                                datafiles: HashMap::default(),
                                parquet_metadata: HashMap::default(),

                                row_count: None,
                            },
                        )
                    })))
            }

            (
                TanicAction::UpdateTable {
                    namespace,
                    table_name,
                    table,
                },
                prev_state,
            ) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    tracing::error!("panic! not connected");
                    panic!();
                };

                let Some(namespacce_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    tracing::error!("panic! ns not found");
                    panic!();
                };

                let Some(ref mut table_desc) = namespacce_desc.tables else {
                    tracing::error!("panic! tables not found");
                    panic!();
                };

                let Some(table_desc) = table_desc.get_mut(&table_name) else {
                    tracing::error!("panic! table desc not found");
                    panic!();
                };

                table_desc.table = Some(table);
            }

            (
                TanicAction::UpdateTableSummary {
                    namespace,
                    table_name,
                    table_summary,
                },
                prev_state,
            ) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    tracing::error!("panic! not connected");
                    panic!();
                };

                let Some(namespace_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    tracing::error!("panic! ns not found");
                    panic!();
                };

                let Some(ref mut table_desc) = namespace_desc.tables else {
                    tracing::error!("panic! tables not found");
                    panic!();
                };

                let Some(table_desc) = table_desc.get_mut(&table_name) else {
                    tracing::error!("panic! table desc not found");
                    panic!();
                };

                if let Some(row_count_str) = table_summary.get(TABLE_SUMMARY_KEY_ROW_COUNT) {
                    if let Ok(row_count) = row_count_str.trim().parse::<u64>() {
                        table_desc.row_count = Some(row_count);
                        tracing::info!(
                            table_row_count = row_count,
                            orig_ns_rows = namespace_desc.row_count,
                            "Bumping NS row count"
                        );
                        namespace_desc.row_count = namespace_desc
                            .row_count
                            .map(|rc| rc + row_count)
                            .or(Some(row_count));
                    }
                }

                table_desc.current_snapshot_summary = Some(table_summary);
            }

            (
                TanicAction::UpdateTableCurrentSnapshot {
                    namespace,
                    table_name,
                    snapshot,
                },
                prev_state,
            ) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    tracing::error!("panic! not connected");
                    panic!();
                };

                let Some(namespacce_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    tracing::error!("panic! ns not found");
                    panic!();
                };

                let Some(ref mut table_desc) = namespacce_desc.tables else {
                    tracing::error!("panic! tble desc not found");
                    panic!();
                };

                let Some(table_desc) = table_desc.get_mut(&table_name) else {
                    tracing::error!("panic! table not found");
                    panic!();
                };

                table_desc.current_snapshot = Some(snapshot);
            }

            (
                TanicAction::UpdateTableCurrentManifestList {
                    namespace,
                    table_name,
                    manifest_list,
                },
                prev_state,
            ) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    tracing::error!("panic! not connected");
                    panic!();
                };

                let Some(namespacce_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    tracing::error!("panic!ns not found");
                    panic!();
                };

                let Some(ref mut table_desc) = namespacce_desc.tables else {
                    tracing::error!("panic! tables not found");
                    panic!();
                };

                let Some(table_desc) = table_desc.get_mut(&table_name) else {
                    tracing::error!("panic! table not found");
                    panic!();
                };

                table_desc.current_manifest_list = Some(*manifest_list);
            }

            (
                TanicAction::UpdateTableManifest {
                    namespace,
                    table_name,
                    manifest,
                    file_path: uri,
                },
                prev_state,
            ) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    tracing::error!("panic! not connected");
                    panic!();
                };

                let Some(namespacce_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    panic!();
                };

                let Some(ref mut table_desc) = namespacce_desc.tables else {
                    panic!();
                };

                let Some(table_desc) = table_desc.get_mut(&table_name) else {
                    panic!();
                };

                table_desc.manifests.insert(uri, *manifest);
            }

            (
                TanicAction::UpdateTableDataFile {
                    namespace,
                    table_name,
                    data_file,
                },
                prev_state,
            ) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    panic!();
                };

                let Some(namespacce_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    panic!();
                };

                let Some(ref mut table_desc) = namespacce_desc.tables else {
                    panic!();
                };

                let Some(table_desc) = table_desc.get_mut(&table_name) else {
                    panic!();
                };

                table_desc
                    .datafiles
                    .insert(data_file.file_path().to_string(), *data_file);
            }

            (
                TanicAction::UpdateTableParquetMetaData {
                    namespace,
                    table_name,
                    file_path,
                    metadata,
                },
                prev_state,
            ) => {
                let TanicAppState { iceberg, .. } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    panic!();
                };

                let Some(namespacce_desc) =
                    retrieved_iceberg_metadata.namespaces.get_mut(&namespace)
                else {
                    panic!();
                };

                let Some(ref mut table_desc) = namespacce_desc.tables else {
                    panic!();
                };

                let Some(table_desc) = table_desc.get_mut(&table_name) else {
                    panic!();
                };

                table_desc.parquet_metadata.insert(file_path, *metadata);
            }

            (TanicAction::FocusPrevNamespace, prev_state) => {
                let TanicAppState { iceberg, ui } = prev_state;

                let TanicUiState::ViewingNamespacesList(ref mut viewing_namespaces_list_state) = ui
                else {
                    panic!();
                };

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    panic!();
                };

                viewing_namespaces_list_state.selected_idx = viewing_namespaces_list_state
                    .selected_idx
                    .map(|selected_idx| {
                        if selected_idx == 0 {
                            retrieved_iceberg_metadata.namespaces.len() - 1
                        } else {
                            selected_idx - 1
                        }
                    });
            }

            (TanicAction::FocusNextNamespace, prev_state) => {
                let TanicAppState { iceberg, ui } = prev_state;

                let TanicUiState::ViewingNamespacesList(ref mut viewing_namespaces_list_state) = ui
                else {
                    panic!();
                };

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    panic!();
                };

                viewing_namespaces_list_state.selected_idx = viewing_namespaces_list_state
                    .selected_idx
                    .map(|selected_idx| {
                        if selected_idx == retrieved_iceberg_metadata.namespaces.len() - 1 {
                            0
                        } else {
                            selected_idx + 1
                        }
                    });
            }

            (TanicAction::SelectNamespace, prev_state) => {
                let TanicAppState { ui, .. } = prev_state;

                let TanicUiState::ViewingNamespacesList(namespaces) = ui else {
                    panic!();
                };

                self.ui = TanicUiState::ViewingTablesList(ViewingTablesListState {
                    namespaces: namespaces.clone(),
                    selected_idx: None,
                });
            }

            (TanicAction::FocusPrevTable, prev_state) => {
                let TanicAppState { iceberg, ui } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    panic!();
                };

                let TanicUiState::ViewingTablesList(ref mut viewing_tables_list_state) = ui else {
                    panic!();
                };

                let Some(namespace_selected_idx) =
                    viewing_tables_list_state.namespaces.selected_idx
                else {
                    panic!();
                };

                let Some(&namespace_selected_name) = retrieved_iceberg_metadata
                    .namespaces
                    .keys()
                    .collect::<Vec<_>>()
                    .get(namespace_selected_idx)
                else {
                    panic!();
                };

                let Some(namespace) = retrieved_iceberg_metadata
                    .namespaces
                    .get(namespace_selected_name)
                else {
                    panic!();
                };

                if let Some(ref table_list) = namespace.tables {
                    let table_list_len = table_list.len();

                    viewing_tables_list_state.selected_idx =
                        viewing_tables_list_state.selected_idx.map(|selected_idx| {
                            if selected_idx == 0 {
                                table_list_len - 1
                            } else {
                                selected_idx - 1
                            }
                        });
                }
            }

            (TanicAction::FocusNextTable, prev_state) => {
                let TanicAppState { iceberg, ui } = prev_state;

                let TanicIcebergState::Connected(ref mut retrieved_iceberg_metadata) = iceberg
                else {
                    panic!();
                };

                let TanicUiState::ViewingTablesList(ref mut viewing_tables_list_state) = ui else {
                    panic!();
                };

                let Some(namespace_selected_idx) =
                    viewing_tables_list_state.namespaces.selected_idx
                else {
                    panic!();
                };

                let Some(&namespace_selected_name) = retrieved_iceberg_metadata
                    .namespaces
                    .keys()
                    .collect::<Vec<_>>()
                    .get(namespace_selected_idx)
                else {
                    panic!();
                };

                let Some(namespace) = retrieved_iceberg_metadata
                    .namespaces
                    .get(namespace_selected_name)
                else {
                    panic!();
                };

                if let Some(ref table_list) = namespace.tables {
                    viewing_tables_list_state.selected_idx =
                        viewing_tables_list_state.selected_idx.map(|selected_idx| {
                            if selected_idx == table_list.len() - 1 {
                                0
                            } else {
                                selected_idx + 1
                            }
                        });
                }
            }

            (TanicAction::Escape, TanicAppState { ui, .. }) => {
                #[allow(clippy::single_match)] // remove once more than one match below
                match ui {
                    TanicUiState::ViewingTablesList(ViewingTablesListState {
                        namespaces, ..
                    }) => self.ui = TanicUiState::ViewingNamespacesList(namespaces.clone()),

                    // TODO: Escape from Partition and DataFile
                    _ => {}
                }
            }
            // TODO:

            // * SelectTable
            // * FocusNextPartition
            // * FocusPrevPartition,
            // * SelectPartition,
            // * FocusNextDataFile,
            // * FocusPrevDataFile,
            // * SelectDataFile
            _ => {
                unimplemented!()
            }
        }

        self
    }
}
