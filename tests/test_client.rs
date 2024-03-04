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
