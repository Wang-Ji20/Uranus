use anyhow::Result;
use tokio::net::TcpListener;

const DEFAULT_PORT: u16 = 12322;

#[tokio::main]
pub async fn main() {
    smain().await.unwrap();
}

async fn smain() -> Result<()> {
    setup_logging()?;
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", DEFAULT_PORT)).await?;
    uranus_s::run(listener).await;
    Ok(())
}

fn setup_logging() -> Result<()> {
    tracing_subscriber::fmt::try_init().map_err(|err| anyhow::anyhow!(err))
}
