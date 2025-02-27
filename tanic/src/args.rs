use clap::Parser;
use http::Uri;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// URI of an Iceberg Catalog to connect to
    pub catalogue_uri: Option<Uri>,
}
