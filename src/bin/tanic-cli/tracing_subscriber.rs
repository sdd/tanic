use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::*;

pub(crate) fn init() {
    tracing::subscriber::set_global_default(
        Registry::default()
            .with(
                EnvFilter::builder()
                    .with_env_var("TANIC_LOG")
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .with(tracing_subscriber::fmt::layer().compact()), //  .pretty())
    )
    .expect("Unable to set global subscriber");
}