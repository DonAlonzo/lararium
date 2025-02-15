use crate::*;

#[derive(Clone)]
pub struct Client {
    host: String,
    port: u16,
    client: reqwest::Client,
}

impl Client {
    pub fn connect(
        host: impl Into<String>,
        port: u16,
    ) -> Self {
        Self {
            host: host.into(),
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
            .post(self.url(false, "/join"))
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
