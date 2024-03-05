use std::net::SocketAddr;

use tokio::{net::TcpListener, task::JoinHandle};

const TEST_ADDR: &str = "127.0.0.1:0";

async fn start_server() -> (SocketAddr, JoinHandle<()>) {
    let listener = TcpListener::bind(TEST_ADDR).await.unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = tokio::spawn(async move { uranus_s::run(listener).await });
    (addr, handle)
}

#[tokio::test]
async fn echo_test() {
    let (addr, _handle) = start_server().await;
    let mut client = uranus_c::Client::connect(addr).await.unwrap();
    let pong = client.echo("hello").await.unwrap();
    assert_eq!("hello", pong);
}

#[tokio::test]
async fn getset_hashmap_test() {
    _ = tracing_subscriber::fmt::try_init();
    let (addr, _handle) = start_server().await;
    let mut client = uranus_c::Client::connect(addr).await.unwrap();
    client.set("hello", "world").await.unwrap();
    let result = client.get("hello").await.unwrap();
    println!("{:?}", result);
}
