use crate::Result;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn listen(&self) -> Result<()> {
        loop {}
    }
}
