use anyhow::Result;
use uranus_c::Client;

const HELLO: &str = "Welcome to uranus client";

#[tokio::main]
async fn main() {
    cmain().await.expect("Error");
}

async fn cmain() -> Result<()> {
    tracing_subscriber::fmt::try_init().unwrap();
    println!("{}", HELLO);
    let mut client = Client::connect("127.0.0.1:12322").await?;
    client.echo("PING").await?;
    println!("uranus connected and pinged the server");
    Ok(())
}
