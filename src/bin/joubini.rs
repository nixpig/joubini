use nanoid::nanoid;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{
    filter::targets::Targets, layer::SubscriberExt, util::SubscriberInitExt,
};

#[derive(Debug)]
struct Proxy {
    local_path: String,
    remote_path: String,
    remote_port: u16,
}

impl Proxy {
    fn new(proxy_config: String) -> Self {
        if let Some((local, remote)) = proxy_config.split_once(':') {
            let (port, path) = if let Some((p1, p2)) = remote.split_once('/') {
                (p1, p2)
            } else {
                (remote, "")
            };

            return Proxy {
                local_path: ["/", local].join(""),
                remote_path: ["/", path].join(""),
                remote_port: port.parse::<u16>().unwrap(),
            };
        }

        error!("Invalid configuration: '{proxy_config}'");
        panic!();
    }
}

#[tokio::main]
async fn main() {
    // let p1 = Proxy::new(String::from(":3000"));
    // let p2 = Proxy::new(String::from("api:3001/api"));
    // let p3 = Proxy::new(String::from("admin:3002/dashboard"));
    // let p4 = Proxy::new(String::from("db:3003"));
    // let p5 = Proxy::new(String::from("deep:3004/deep/nested/path"));

    let filter_layer = Targets::from_str(
        std::env::var("RUST_LOG").as_deref().unwrap_or("info"),
    )
    .unwrap();

    let format_layer = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();

    info!("Starting...");

    let addr = "127.0.0.1:7878";

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            info!("Bound to address");
            l
        }
        Err(_) => {
            error!("Unable to bind to port");
            panic!();
        }
    };

    loop {
        let uid = nanoid!(5);

        if let Ok(socket) = listener.accept().await {
            info!("[{uid}] Established TCP connection");

            tokio::spawn(async move {
                info!("[{uid}] Handling request");

                let mut proxies = HashMap::new();

                proxies.insert("zero", (3000, ""));
                proxies.insert("one", (3001, ""));

                let (stream, _) = socket;

                if handle_connection(stream, &proxies, &uid).await.is_err() {
                    error!("[{uid}] Unable to handle connection");
                }
            });
        } else {
            error!("[{uid}] Unable to establish a TCP connection");
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    proxies: &HashMap<&str, (i32, &str)>,
    uid: &String,
) -> Result<(), Box<dyn Error>> {
    let mut buf_reader = BufReader::new(&mut stream);

    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line).await?;

    let mut headers: HashMap<String, String> = HashMap::new();
    while let Some(line) = buf_reader.lines().next_line().await? {
        if line.is_empty() {
            break;
        }

        let (header_name, header_value) = line
            .split_once(':')
            .expect("headers must have a name and a value");
        headers.insert(
            String::from(header_name.trim()),
            String::from(header_value.trim()),
        );
    }

    let mut body: Vec<String> = vec![];

    // let proxy_port = proxies[base].0;
    //
    // if let Ok(mut remote_stream) =
    //     TcpStream::connect(format!("localhost:{}", proxy_port).as_str()).await
    // {
    //     info!("[{uid}] Connected to remote");
    //
    //     println!("{:?}", request);
    //
    //     let mut request = request.replace(
    //         "localhost:7878",
    //         format!("localhost:{}", proxy_port).as_str(),
    //     );
    //     request.push_str("\r\nConnection: close\r\n\r\n");
    //
    //     remote_stream.write_all(request.as_bytes()).await?;
    //
    //     let mut res = vec![];
    //     remote_stream.read_to_end(&mut res).await?;
    //
    //     stream.write_all(&res).await?;
    // } else {
    //     warn!("[{uid}] Unable to connect to remote");
    // };
    //
    // if stream.flush().await.is_err() {
    //     warn!("Unable to flush stream");
    // }

    Ok(())
}
