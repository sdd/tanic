//! Iceberg Context

use iceberg::Catalog;
use iceberg_catalog_rest::{RestCatalog, RestCatalogConfig};
use std::sync::{Arc, RwLock};
use tanic_core::config::ConnectionDetails;
use tanic_core::message::NamespaceDeets;
use tanic_core::Result;

/// Iceberg Context
#[derive(Debug)]
pub struct IcebergContext {
    conn_details: ConnectionDetails,

    namespaces: Arc<RwLock<Vec<NamespaceDeets>>>,

    /// Iceberg Catalog
    catalog: Arc<dyn Catalog>,
}

impl IcebergContext {
    /// Create a new Iceberg Context from a Uri
    pub fn from_connection_details(conn_details: &ConnectionDetails) -> Self {
        let conn_details = conn_details.clone();

        let mut uri_str = conn_details.uri.to_string();
        uri_str.pop();

        let config = RestCatalogConfig::builder().uri(uri_str).build();
        let rest_catalog = RestCatalog::new(config);

        Self {
            conn_details,
            namespaces: Arc::new(RwLock::new(vec![])),
            catalog: Arc::new(rest_catalog),
        }
    }

    pub async fn populate_namespaces(&self) -> Result<Arc<RwLock<Vec<NamespaceDeets>>>> {
        let namespaces = self.namespaces.clone();
        let catalog = self.catalog.clone();

        let root_namespaces = catalog.list_namespaces(None).await.unwrap();

        let root_namespaces = root_namespaces
            .into_iter()
            .map(|ns| {
                let parts = ns.inner();
                let name = parts.clone().join(".");
                NamespaceDeets {
                    parts,
                    name,
                    table_count: 1,
                }
            })
            .collect::<Vec<_>>();

        {
            let mut guard = namespaces.write().unwrap();
            *guard = root_namespaces;
        }

        Ok(namespaces)
    }

    pub fn namespaces(&self) -> Arc<RwLock<Vec<NamespaceDeets>>> {
        self.namespaces.clone()
    }
}
