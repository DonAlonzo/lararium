pub struct Cli {
    api: api::Client,
}

impl Cli {
    pub fn connect(
        host: String,
        port: u16,
    ) -> Self {
        Self {
            api: api::Client::connect(host, port),
        }
    }
}
