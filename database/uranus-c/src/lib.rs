use anyhow::{anyhow, Result};
use bytes::Bytes;
use thiserror::Error;
use tokio::net::{TcpStream, ToSocketAddrs};
use tracing::debug;
use uranus_s::{Connection, Echo, Frame, Get, Put};

pub struct Client {
    connection: Connection,
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("Connection reset by the server. Maybe the server is closed.")]
    ConnectionReset,
    #[error("The response is illegal.")]
    BadResponse,
    #[error("Unexpected frame")]
    UnexpectedFrame(String),
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

    pub async fn get(&mut self, key: &str) -> Result<Option<Bytes>> {
        let frame = Get::new(key).into_frame();
        debug!(request = ?frame);
        self.connection.write_frame(&frame).await?;
        match self.read_response().await? {
            Frame::Text(txt) => Ok(Some(txt.into())),
            Frame::Binary(binary) => Ok(Some(binary)),
            Frame::Null => Ok(None),
            frame => Err(ClientError::UnexpectedFrame(format!("{}", frame)))?,
        }
    }

    pub async fn set(&mut self, key: &str, value: impl Into<Bytes>) -> Result<()> {
        let frame = Put::new(key.to_owned(), value.into()).into_frame();
        debug!(request = ?frame);
        self.connection.write_frame(&frame).await?;
        match self.read_response().await? {
            Frame::Text(txt) if txt == "OK" => Ok(()),
            frame => Err(ClientError::UnexpectedFrame(format!("{}", frame)))?,
        }
    }
}
