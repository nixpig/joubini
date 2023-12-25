use std::net::TcpListener;

fn main() {
    let host = "127.0.0.1";
    let port = "80";

    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();

    for stream in listener.incoming() {
        println!("Hello hello hello hi!");
        println!("{:?}", stream);
    }
}
