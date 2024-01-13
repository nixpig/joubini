use std::sync::Arc;

use joubini::{
    error::{Error, IoError},
    server::start,
    settings::get_settings,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Error> {
    match get_settings(std::env::args_os().collect()) {
        Ok(settings) => {
            let settings = Arc::new(settings);

            match TcpListener::bind(format!(
                "{}:{}",
                settings.host, settings.local_port
            ))
            .await
            {
                Ok(listener) => {
                    let listener = Arc::new(listener);
                    start(listener.clone(), settings.clone()).await
                }
                Err(e) => {
                    eprintln!("Unable to bind to local port");
                    Err(Error::IoError(IoError::StdIo(e)))
                }
            }
        }
        Err(e) => {
            eprintln!("Unable to get settings");
            Err(e)
        }
    }
}
