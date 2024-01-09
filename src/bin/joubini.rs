use std::error::Error;

use joubini::{server::start, settings::get_settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = get_settings();
    println!("{:#?}", settings);

    start(settings).await
}
