//! Tanic Config module

use directories::ProjectDirs;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use http::Uri;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Result;

/// Represents a named set of connection details for an Iceberg catalog
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionDetails {
    pub id: Uuid,

    /// The name of this connection
    pub name: String,

    /// The URI of this connection
    #[serde(with = "http_serde::uri")]
    pub uri: Uri,
    // Type?
}

impl ConnectionDetails {
    pub fn new_anon(uri: Uri) -> Self {
        let mut generator = names::Generator::default();

        Self {
            id: Uuid::new_v4(),
            name: generator.next().expect("could not generate a random name"),
            uri,
        }
    }
}

impl PartialEq for ConnectionDetails {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
    }
}

/// persistable user config.
///
/// Loaded in at application startup from $CONFIG/tanic/tanic.toml
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TanicConfig {
    /// list of known connections
    pub library: Vec<ConnectionDetails>,
}

impl TanicConfig {
    /// Load config by merging standard sources of config
    ///
    /// Priority: defaults < config file < env < args
    pub fn load() -> Result<TanicConfig> {
        let mut figment = Figment::from(Serialized::defaults(TanicConfig::default()));

        if let Some(proj_dirs) = ProjectDirs::from("com", "Tanic", "Tanic") {
            let config_dir = proj_dirs.config_dir();

            figment = figment.merge(Toml::file(config_dir.join("tanic.toml")))
        }

        Ok(figment.merge(Env::prefixed("TANIC_")).extract()?)
    }
}
