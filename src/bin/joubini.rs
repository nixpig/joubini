use std::{error::Error, sync::Arc};

use joubini::{server::start, settings::get_settings};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Arc::new(get_settings());

    let listener = Arc::new(
        TcpListener::bind(format!(
            "127.0.0.1:{}",
            settings.local_port.unwrap()
        ))
        .await?,
    );

    start(listener.clone(), settings.clone()).await
}
