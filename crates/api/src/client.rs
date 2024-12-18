use crate::*;
use reqwest::StatusCode;

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

    pub async fn get(
        &self,
        topic: &Topic,
    ) -> Result<Entry> {
        let response = self
            .client
            .get(self.url(false, &format!("/~/{topic}")))
            .send()
            .await
            .unwrap();

        if !response.status().is_success() {
            return match response.status() {
                StatusCode::NOT_FOUND => Err(Error::NotFound),
                _ => Err(Error::Unknown),
            };
        }

        Ok(response.json().await.unwrap())
    }
}
