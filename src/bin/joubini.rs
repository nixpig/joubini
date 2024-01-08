use joubini::settings::get_settings;

#[tokio::main]
async fn main() {
    println!("\n");
    let settings = get_settings();
    println!("count: {}", settings.proxies.len());
}
