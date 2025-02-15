use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::*;

#[allow(unused)]
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

pub(crate) fn init_tui_logger(no_ui: bool) {
    if no_ui {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    } else {
        tracing_subscriber::registry()
            // .with(console_subscriber::spawn())
            .with(tui_logger::tracing_subscriber_layer())
            .init();

        tui_logger::init_logger(tui_logger::LevelFilter::Trace)
            .expect("Could not initialize logger");
    }
}
