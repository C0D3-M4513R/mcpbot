use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;

mod client;

mod codegen {
    include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
}

fn main() -> anyhow::Result<()> {
    let stdout = tracing_subscriber::fmt::Layer::default();

    let subscriber = tracing_subscriber::registry::Registry::default() // provide underlying span data store
        .with(LevelFilter::INFO) // filter out low-level debug tracing (eg tokio executor)
        .with(stdout); // log to stdout

    tracing::subscriber::set_global_default(subscriber).expect("setting global default failed");

    dotenvy::dotenv().expect("Could not find .env");

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Could not build runtime");

    let _enter = runtime.enter();
    let _ = runtime.block_on(client::init_client())?;

    Ok(())
}