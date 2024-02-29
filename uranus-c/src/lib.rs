use anyhow::Result;
use bytes::Bytes;
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::debug;
use uranus_s::{Connection, Frame};

pub struct Client {
    connection: Connection,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
        let socket = TcpStream::connect(addr).await?;
        let connection = Connection::new(socket);
        Ok(Client { connection })
    }

    pub async fn ping(&mut self, _: Option<Bytes>) -> Result<()> {
        let frame = Frame::Text("PING".to_string());
        debug!(request = ?frame);
        self.connection.write_frame(&frame).await?;
        Ok(())
    }
}
