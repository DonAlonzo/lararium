use bytes::{Bytes, BytesMut};
use std::sync::{Arc, RwLock};
use wasmtime_wasi::{async_trait, HostOutputStream, StdoutStream, StreamError, Subscribe};

pub struct StdOut;

struct StdOutStream {
    buffer: Arc<RwLock<BytesMut>>,
}

impl StdOut {
    pub fn new() -> Self {
        Self
    }
}

impl StdOutStream {
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(RwLock::new(BytesMut::with_capacity(1024 * 1024))),
        }
    }
}

impl StdoutStream for StdOut {
    fn stream(&self) -> Box<dyn HostOutputStream> {
        Box::new(StdOutStream::new())
    }

    fn isatty(&self) -> bool {
        false
    }
}

impl HostOutputStream for StdOutStream {
    fn write(
        &mut self,
        bytes: Bytes,
    ) -> Result<(), StreamError> {
        let mut buf = self.buffer.write().unwrap();
        buf.extend_from_slice(bytes.as_ref());
        Ok(())
    }

    fn flush(&mut self) -> Result<(), StreamError> {
        let mut buf = self.buffer.write().unwrap();
        if let Some(pos) = buf.iter().position(|&b| b == b'\n') {
            let extracted = buf.split_to(pos + 1);
            let Ok(line) = std::str::from_utf8(&extracted) else {
                return Ok(());
            };
            println!("{}", line.trim_end());
        }
        Ok(())
    }

    fn check_write(&mut self) -> Result<usize, StreamError> {
        Ok(1024 * 1024)
    }
}

#[async_trait]
impl Subscribe for StdOutStream {
    async fn ready(&mut self) {}
}
