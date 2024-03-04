use anyhow::{anyhow, Result};
use thiserror::Error;
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::debug;
use uranus_s::{Connection, Echo, Frame};

pub struct Client {
    connection: Connection,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Connection reset by the server. Maybe the server is closed.")]
    ConnectionReset,
    #[error("The response is illegal.")]
    BadResponse,
}

impl Client {
    pub async fn connect<T: ToSocketAddrs>(addr: T) -> Result<Client> {
        let socket = TcpStream::connect(addr).await?;
        let connection = Connection::new(socket);
        Ok(Client { connection })
    }

    /// Send an echo message to the server.
    /// returns the echoed message, don't check the correctness.
    /// PING is implemented by echo
    pub async fn echo(&mut self, echo: impl ToString) -> Result<String> {
        let frame = Echo::new(echo).into_frame();
        self.connection.write_frame(&frame).await?;
        match self.read_response().await? {
            Frame::Text(txt) => Ok(txt),
            _ => Err(ClientError::BadResponse)?,
        }
    }

    /// Reads a message from socket.
    async fn read_response(&mut self) -> Result<Frame> {
        let response = self.connection.read_frame().await?;
        debug!(?response);
        match response {
            Some(Frame::Error(err)) => Err(anyhow!(err)),
            Some(frame) => Ok(frame),
            None => Err(ClientError::ConnectionReset)?,
        }
    }
}
