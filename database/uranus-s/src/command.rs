use std::vec;

use crate::Connection;

use super::Frame;
use anyhow::Result;
use bytes::Bytes;
use thiserror::Error;

/// [`Command`] is a semantic information atom between client and server.
#[derive(Debug)]
pub enum Command {
    Set(Set),
    Get(Get),
    Echo(Echo),
}

impl Command {
    /// Parse a command from network frames
    /// This function is usually called by the server to understand
    /// what client wants to do.
    pub fn from_frame(frame: Frame) -> Result<Command> {
        let mut parser = CommandParser::new(frame)?;
        let command_name = parser
            .next_string()?
            .ok_or(CommandParseError::UnexpectedEOF)?
            .to_lowercase();
        let command = match command_name.as_str() {
            "get" => Command::Get(Get::parse_frames(&mut parser)?),
            "set" => Command::Set(Set::parse_frames(&mut parser)?),
            "echo" => Command::Echo(Echo::parse_frames(&mut parser)?),
            _ => Err(CommandParseError::UnknownCommand)?,
        };
        parser.exhausted()?;
        Ok(command)
    }

    pub async fn apply(self, dst: &mut Connection) -> Result<()> {
        use Command::*;

        match self {
            Echo(echo) => echo.apply(dst).await,
            _ => todo!(),
        }
    }
}

/// This struct parses the command from network frames, remembering current cursor position.
pub struct CommandParser {
    tokens: vec::IntoIter<Frame>,
}

#[derive(Debug, Error)]
pub enum CommandParseError {
    UnexpectedEOF,
    ArgNotArray,
    ArgNotText,
    ArgNotBinary,
    UnexpectedFrame,
    UnknownCommand,
}

impl std::fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandParseError::UnexpectedEOF => {
                write!(f, "protocol requires more frames, but not given.")
            }
            CommandParseError::ArgNotArray => write!(
                f,
                "protocol requires that all commands are arrays, but this is not an array type."
            ),
            CommandParseError::ArgNotText => {
                write!(f, "protocol expects a text frame, but this is not.")
            }
            CommandParseError::ArgNotBinary => {
                write!(f, "protocol expects a binary frame, but this is not")
            }
            CommandParseError::UnexpectedFrame => write!(
                f,
                "the args should be enough, but there's one more frame left."
            ),
            CommandParseError::UnknownCommand => {
                write!(f, "The command is not implemented in this system.")
            }
        }
    }
}

impl CommandParser {
    /// The command is always an array of frames
    /// Even if the command don't have any arguments, it is put in an array as still.
    pub fn new(frame: Frame) -> Result<CommandParser> {
        let Frame::Array(array) = frame else {
            Err(CommandParseError::ArgNotArray)?
        };
        Ok(CommandParser {
            tokens: array.into_iter(),
        })
    }

    fn next(&mut self) -> Option<Frame> {
        self.tokens.next()
    }

    pub fn next_string(&mut self) -> Result<Option<String>> {
        if let Some(frame) = self.next() {
            match frame {
                Frame::Text(txt) => Ok(Some(txt)),
                Frame::Binary(binary) => {
                    std::str::from_utf8(&binary).map(|s| Ok(Some(s.to_string())))?
                }
                _ => Err(CommandParseError::ArgNotText)?,
            }
        } else {
            Ok(None)
        }
    }

    pub fn next_bytes(&mut self) -> Result<Option<Bytes>> {
        if let Some(frame) = self.next() {
            match frame {
                Frame::Binary(binary) => Ok(Some(binary)),
                Frame::Text(txt) => Ok(Some(Bytes::from(txt.into_bytes()))),
                _ => Err(CommandParseError::ArgNotBinary)?,
            }
        } else {
            Ok(None)
        }
    }

    pub fn exhausted(&mut self) -> Result<()> {
        if self.tokens.next().is_none() {
            Ok(())
        } else {
            Err(CommandParseError::UnexpectedFrame)?
        }
    }
}

/// This command set `key` to hold a value `value`.
/// if `key` already have a value, that value is overwritten,
#[derive(Debug)]
pub struct Set {
    pub key: String,
    pub value: Bytes,
}

impl Set {
    pub fn new(key: impl ToString, value: Bytes) -> Set {
        Set {
            key: key.to_string(),
            value,
        }
    }

    pub fn parse_frames(parser: &mut CommandParser) -> Result<Set> {
        let key = parser
            .next_string()?
            .ok_or(CommandParseError::UnexpectedEOF)?;
        let value = parser
            .next_bytes()?
            .ok_or(CommandParseError::UnexpectedEOF)?;
        Ok(Set { key, value })
    }

    /// Consume this command to generate an array frame representation
    pub fn into_frame(self) -> Frame {
        let frame = vec![
            Frame::Text("set".to_string()),
            Frame::Text(self.key),
            Frame::Binary(self.value),
        ];
        Frame::Array(frame)
    }
}

/// If the key does not exists, returns nil. otherwise just normal.
#[derive(Debug)]
pub struct Get {
    pub key: String,
}

impl Get {
    pub fn new(key: impl ToString) -> Get {
        Get {
            key: key.to_string(),
        }
    }

    pub fn parse_frames(parser: &mut CommandParser) -> Result<Get> {
        let key = parser
            .next_string()?
            .ok_or(CommandParseError::UnexpectedEOF)?;
        Ok(Get { key })
    }

    pub fn into_frame(self) -> Frame {
        let frame = vec![Frame::Text("get".to_string()), Frame::Text(self.key)];
        Frame::Array(frame)
    }
}

#[derive(Debug)]
pub struct Echo {
    pub echo: String,
}

impl Echo {
    pub fn new(echo: impl ToString) -> Echo {
        Echo {
            echo: echo.to_string(),
        }
    }

    pub fn parse_frames(parser: &mut CommandParser) -> Result<Echo> {
        let echo = parser
            .next_string()?
            .ok_or(CommandParseError::UnexpectedEOF)?;
        Ok(Echo { echo })
    }

    pub async fn apply(self, dst: &mut Connection) -> Result<()> {
        let response = Frame::Text(self.echo);
        dst.write_frame(&response).await?;
        Ok(())
    }

    pub fn into_frame(self) -> Frame {
        let frame = vec![Frame::Text("echo".to_string()), Frame::Text(self.echo)];
        Frame::Array(frame)
    }
}
