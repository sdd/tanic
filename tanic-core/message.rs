use crate::config::ConnectionDetails;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
#[non_exhaustive]
pub enum TanicMessage {
    Exit,

    ConnectTo(ConnectionDetails),

    // Update of list of namespaces for current location
    ShowNamespaces(Vec<NamespaceDeets>),

    ShowTablesForNamespace {
        namespace: String,
        tables: Vec<TableDeets>,
    },

    NavigateUp,

    NavigateChildNamespace(String),

    TableNameList(String),
}

#[derive(Clone, Debug)]
pub struct NamespaceDeets {
    pub parts: Vec<String>,
    pub name: String,
    pub table_count: usize,
}

#[derive(Clone, Debug)]
pub struct TableDeets {
    pub namespace: Vec<String>,
    pub name: String,
    pub row_count: usize,
}
