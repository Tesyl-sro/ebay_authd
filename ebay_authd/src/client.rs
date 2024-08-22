use crate::error::Result;
use ebay_authd_core::Message;
use log::debug;
use std::{
    io::{BufRead, BufReader, BufWriter, Write},
    os::{
        fd::{AsRawFd, BorrowedFd},
        unix::net::UnixStream,
    },
};

pub struct Client {
    reader: BufReader<UnixStream>,
    writer: BufWriter<UnixStream>,
}

impl Client {
    pub fn new(stream: UnixStream) -> Result<Self> {
        let copy = stream.try_clone()?;

        Ok(Self {
            reader: BufReader::new(stream),
            writer: BufWriter::new(copy),
        })
    }

    pub fn await_message(&mut self) -> Result<Option<Message>> {
        debug!("Waiting for message from client");
        let mut buffer = String::with_capacity(64);

        if self.reader.read_line(&mut buffer)? == 0 {
            return Ok(None);
        }

        buffer.pop();
        debug!("Received: {buffer}");
        let message = Message::deserialize(&buffer)?;

        Ok(Some(message))
    }

    pub fn message<M: Into<Message>>(&mut self, message: M) -> Result<()> {
        let message = message.into();
        let json = message.serialize()?;
        debug!("Sending: {json}");

        self.writer.write_all(json.as_bytes())?;
        self.writer.write_all(b"\n")?;
        self.writer.flush()?;

        Ok(())
    }

    pub fn as_raw_fd(&self) -> i32 {
        self.reader.get_ref().as_raw_fd()
    }
}

impl<'f> PartialEq<BorrowedFd<'f>> for Client {
    fn eq(&self, other: &BorrowedFd) -> bool {
        let other = other.as_raw_fd();
        let this = self.as_raw_fd();

        other == this
    }
}
