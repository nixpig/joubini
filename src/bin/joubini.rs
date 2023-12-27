use nanoid::nanoid;
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

#[tokio::main]
async fn main() {
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

                let (stream, _) = socket;

                if handle_connection(stream, &uid).await.is_err() {
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
    uid: &String,
) -> Result<(), Box<dyn Error>> {
    let buf_reader = BufReader::new(&mut stream);

    let mut request: Vec<_> = vec![];

    let mut lines = buf_reader.lines();

    while let Some(line) = lines.next_line().await? {
        if !line.is_empty() {
            request.push(line);
        } else {
            break;
        }
    }

    let request = request.join("\r\n");

    let sleep = "GET /sleep HTTP/1.1\r\n";

    let (status_line, contents) = if request.starts_with(sleep) {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        ("HTTP/1.1 200 OK\r\n\r\n", "Slept")
    } else {
        if let Ok(mut remote_stream) =
            TcpStream::connect("jsonplaceholder.typicode.com:80").await
        {
            info!("[{uid}] Connected to jsonplaceholder");

            println!("{:?}", request);

            let mut request = request
                .replace("localhost:7878", "jsonplaceholder.typicode.com");
            request.push_str("\r\nConnection: close\r\n\r\n");

            println!("replaced: {}", request);

            remote_stream.write_all(request.as_bytes()).await?;

            let mut res = vec![];
            remote_stream.read_to_end(&mut res).await?;
            println!("read: {:#?}", std::str::from_utf8(&res)?);

            stream.write_all(&res).await?;
        } else {
            warn!("[{uid}] Unable to connect to jsonplaceholder");
        }

        ("HTTP/1.1 200 OK\r\n\r\n", "Hello, world!")
    };

    let response = format!("{status_line}{contents}");

    if stream.write_all(response.as_bytes()).await.is_err() {
        warn!("[{uid}] Unable to write to stream");
    }

    if stream.flush().await.is_err() {
        warn!("Unable to flush stream");
    }

    Ok(())
}
