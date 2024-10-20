use super::{
    ash::Ash,
    frame::{BufExt, BufMutExt, Frame},
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio::sync::watch;

#[derive(Clone)]
pub struct Adapter {
    ash: Ash,
}

impl Adapter {
    pub fn new() -> Self {
        Self { ash: Ash::new() }
    }

    pub async fn reset(&mut self) {
        self.ash.reset()
    }

    pub async fn wait_until_ready(&mut self) {
        self.ash.wait_until_ready().await
    }

    pub async fn send_query_version(&mut self) {
        self.ash.send(&[0x00, 0x00, 0x00, 0x02])
    }

    pub async fn send_init_network(&mut self) {
        self.ash.send(&[0x00, 0x00, 0x01, 0x02])
    }

    pub async fn feed(
        &mut self,
        buffer: &[u8],
    ) -> usize {
        self.ash.feed(buffer)
    }

    pub fn poll(&mut self) -> Option<Vec<u8>> {
        self.ash.poll()
    }

    pub async fn poll_async(&mut self) -> Vec<u8> {
        self.ash.poll_async().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let mut adapter = Adapter::new();
        assert_eq!(None, adapter.poll());
    }
}
