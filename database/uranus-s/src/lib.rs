//! Uranus server library & Client-Server interface
//!

pub mod command;
pub use command::*;

pub mod db;
pub use db::*;

use std::{io::Cursor, time::Duration};

use anyhow::{anyhow, Result};
use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
    time,
};
use tracing::{debug, error, info};

pub async fn run(listener: TcpListener) {
    let mut server = Listener {
        listener,
        db: DBHandle::new(),
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
    }
}

/// [`Listener`] listens a port, waiting for connections. Established connection is served by
/// [`Handler`].
#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    db: DBHandle,
}

impl Listener {
    async fn run(&mut self) -> Result<()> {
        info!("uranus started to serve requests");

        loop {
            let socket = self.accept().await?;

            let mut handler = Handler {
                connection: Connection::new(socket),
                _database: self.db.clone(),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
            });
        }
    }

    async fn accept(&mut self) -> Result<TcpStream> {
        let mut backoff = 1;
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;
            backoff *= 2;
        }
    }
}

pub struct Handler {
    connection: Connection,
    _database: DBHandle,
}

impl Handler {
    async fn run(&mut self) -> Result<()> {
        loop {
            let frame = tokio::select! {
                res = self.connection.read_frame() => res?
            };

            let frame = match frame {
                Some(frame) => frame,
                None => return Ok(()),
            };

            info!("received a frame {:?}", frame);

            let cmd = Command::from_frame(frame)?;
            debug!(?cmd);

            cmd.apply(&mut self.connection).await?;
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
}

const BUFFER_SIZE: usize = 4 * 1024;

impl Connection {
    pub fn new(socket: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(socket),
            buffer: BytesMut::with_capacity(BUFFER_SIZE),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }
            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                }
                return Err(anyhow!("connection reset by peer"));
            }
        }
    }

    /// [`write_frame`] can't deal with recursions
    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        match frame {
            Frame::Array(val) => {
                self.stream.write_u8(b'*').await?;
                self.write_decimal(val.len() as u64).await?;
                for entry in val {
                    self.write_scalar(entry).await?;
                }
            }
            _ => self.write_scalar(frame).await?,
        };
        self.stream.flush().await?; // note: the '?' cast io::Error to anyhow::Error
        Ok(())
    }

    pub async fn write_scalar(&mut self, frame: &Frame) -> Result<()> {
        match frame {
            Frame::Text(s) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(s.as_bytes()).await?;
            }
            Frame::Error(err) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(err.as_bytes()).await?;
            }
            Frame::Binary(bin) => {
                let len = bin.len();

                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(bin).await?;
            }
            Frame::Null => todo!(),
            Frame::Array(_) => Err(FrameError::Recursive)?,
        }
        self.write_crlf().await?;
        Ok(())
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);
        match Frame::check(&mut buf) {
            Ok(None) => Ok(None),
            Ok(Some(())) => {
                let len = buf.position() as usize;
                buf.set_position(0);
                let frame = Frame::parse(&mut buf)?.unwrap(); // Frame::check guaranteed Some(_)
                self.buffer.advance(len);
                Ok(Some(frame))
            }
            Err(e) => Err(e),
        }
    }

    async fn write_crlf(&mut self) -> Result<()> {
        self.stream.write_all(b"\r\n").await?;
        Ok(())
    }

    async fn write_decimal(&mut self, val: u64) -> Result<()> {
        use std::io::Write;

        let mut buf = [0u8; 20];
        let mut buf = Cursor::new(&mut buf[..]);
        write!(&mut buf, "{}", val)?;
        let pos = buf.position() as usize;
        self.stream.write_all(&buf.get_ref()[..pos]).await?;
        self.write_crlf().await?;
        Ok(())
    }
}

/// [`Frame`] is a transmission atom between client and server. A command typically
/// consists of many frames. Command may arrange them to arrays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Frame {
    Text(String),
    Error(String),
    Binary(bytes::Bytes),
    Array(Vec<Frame>),
    Null,
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("This frame is incomplete")]
    Incomplete,
    #[error("Uranus wire protocol doesn't support recursive array types")]
    Recursive,
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<Option<()>> {
        match get_u8_bump(src) {
            Some(b'+') => Ok(get_line_bump(src).map(|_| ())),
            Some(b'-') => Ok(get_line_bump(src).map(|_| ())),
            Some(b'*') => {
                let len = get_decimal_bump(src)?;

                for _ in 0..len {
                    Frame::check(src)?;
                }

                Ok(Some(()))
            }
            Some(b'$') => {
                let len: usize = get_decimal_bump(src)?.try_into()?;
                skip(src, len + 2)?;
                Ok(Some(()))
            }
            None => Ok(None),
            _ => unimplemented!(),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Option<Frame>> {
        match get_u8_bump(src) {
            Some(b'+') => {
                if let Some(line) = get_line_bump(src).map(|x| x.to_vec()) {
                    let string = String::from_utf8(line)?;
                    Ok(Some(Frame::Text(string)))
                } else {
                    Ok(None)
                }
            }
            Some(b'-') => {
                let line = get_line_bump(src).ok_or(FrameError::Incomplete)?.to_vec();
                let string = String::from_utf8(line)?;

                Ok(Some(Frame::Error(string)))
            }
            Some(b'*') => {
                let len = get_decimal_bump(src)?.try_into()?;
                let mut out = Vec::with_capacity(len);

                for _ in 0..len {
                    out.push(Frame::parse(src)?.unwrap());
                }

                Ok(Some(Frame::Array(out)))
            }
            Some(b'$') => {
                let len = get_decimal_bump(src)?.try_into()?;
                let n = len + 2;

                if src.remaining() < n {
                    return Err(FrameError::Incomplete)?;
                }

                let data = bytes::Bytes::copy_from_slice(&src.chunk()[..len]);
                skip(src, n)?;
                Ok(Some(Frame::Binary(data)))
            }
            None => Ok(None),
            _ => unimplemented!(),
        }
    }
}

impl std::fmt::Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Frame::Text(txt) => std::fmt::Display::fmt(&txt, f),
            Frame::Error(err) => write!(f, "error: {}", err),
            Frame::Binary(binary) => std::fmt::LowerHex::fmt(&binary, f),
            Frame::Array(parts) => {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }

                    std::fmt::Display::fmt(&part, f)?;
                }
                Ok(())
            }
            Frame::Null => write!(f, "(nil)"),
        }
    }
}

fn get_line_bump<'a>(src: &mut Cursor<&'a [u8]>) -> Option<&'a [u8]> {
    let start = src.position() as usize;
    let end = src.get_ref().len() - 1;
    for i in start..end {
        if src.get_ref()[i] == b'\r' && src.get_ref()[i + 1] == b'\n' {
            src.set_position((i + 2) as u64);
            return Some(&src.get_ref()[start..i]);
        }
    }
    None
}

fn get_u8_bump(src: &mut Cursor<&[u8]>) -> Option<u8> {
    if !src.has_remaining() {
        return None;
    }
    Some(src.get_u8())
}

fn skip(src: &mut Cursor<&[u8]>, n: usize) -> Result<()> {
    if src.remaining() < n {
        Err(FrameError::Incomplete)?
    }

    src.advance(n);
    Ok(())
}

fn get_decimal_bump(src: &mut Cursor<&[u8]>) -> Result<u64> {
    let line = get_line_bump(src).ok_or(FrameError::Incomplete)?;
    let utf8_num = std::str::from_utf8(line)?;
    Ok(utf8_num.parse::<u64>()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_frame() {
        let literal_frame = b"*2\r\n+SET\r\n+123\r\n";
        let mut cursor: Cursor<&[u8]> = Cursor::new(literal_frame);
        let parsed_frame = Frame::parse(&mut cursor).unwrap().unwrap();
        let arr_frames = Frame::Array(vec![
            Frame::Text("SET".to_string()),
            Frame::Text("123".to_string()),
        ]);
        assert_eq!(parsed_frame, arr_frames)
    }
}
