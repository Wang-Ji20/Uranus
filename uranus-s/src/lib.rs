//! Uranus server library & Client-Server interface
//!

use std::{io::Cursor, time::Duration};

use anyhow::{anyhow, Result};
use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::{TcpListener, TcpStream},
    time,
};
use tracing::{error, info};

pub async fn run(listener: TcpListener) -> Result<()> {
    let mut server = Listener { listener };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
    }

    Ok(())
}

/// [`Listener`] listens a port, waiting for connections. Established connection is served by
/// [`Handler`].
#[derive(Debug)]
struct Listener {
    listener: TcpListener,
}

impl Listener {
    async fn run(&mut self) -> Result<()> {
        info!("uranus started to serve requests");

        loop {
            let socket = self.accept().await?;

            let mut handler = Handler {
                connection: Connection::new(socket),
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
}

impl Handler {
    async fn run(&mut self) -> Result<()> {
        let frame = tokio::select! {
            res = self.connection.read_frame() => res?
        };

        let frame = match frame {
            Some(frame) => frame,
            None => return Ok(()),
        };

        info!("received a frame {:?}", frame);
        Ok(())
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

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        match frame {
            Frame::Simple(s) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(s.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
        };
        self.stream.flush().await?; // note: the '?' cast io::Error to anyhow::Error
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
}

#[derive(Clone, Debug)]
pub enum Frame {
    Simple(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FrameError {
    #[error("This frame is incomplete")]
    Incomplete,
}

impl Frame {
    pub fn check(src: &mut Cursor<&[u8]>) -> Result<Option<()>> {
        match get_u8(src) {
            Some(b'+') => Ok(if get_line(src).is_some() {
                Some(())
            } else {
                None
            }),
            None => Ok(None),
            _ => unimplemented!(),
        }
    }

    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Option<Frame>> {
        match get_u8(src) {
            Some(b'+') => {
                if let Some(line) = get_line(src).map(|x| x.to_vec()) {
                    let string = String::from_utf8(line)?;
                    Ok(Some(Frame::Simple(string)))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
            _ => unimplemented!(),
        }
    }
}

fn get_line<'a>(src: &mut Cursor<&'a [u8]>) -> Option<&'a [u8]> {
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

fn get_u8(src: &mut Cursor<&[u8]>) -> Option<u8> {
    if !src.has_remaining() {
        return None;
    }
    Some(src.get_u8())
}
