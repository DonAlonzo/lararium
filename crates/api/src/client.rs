use crate::*;

pub struct Client {
    host: String,
    port: u16,
    client: reqwest::Client,
}

impl Client {
    pub fn connect(
        host: String,
        port: u16,
    ) -> Self {
        Self {
            host,
            port,
            client: reqwest::Client::new(),
        }
    }

    fn url(
        &self,
        secure: bool,
        path: &str,
    ) -> String {
        format!(
            "{}://{}:{}{}",
            if secure { "https" } else { "http" },
            self.host,
            self.port,
            path,
        )
    }

    pub async fn join(
        &self,
        request: JoinRequest,
    ) -> Result<JoinResponse> {
        let response = self
            .client
            .post(self.url(false, JOIN_PATH))
            .json(&request)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        Ok(response)
    }
}
