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
        key: &Key,
    ) -> Result<Entry> {
        let response = self
            .client
            .get(self.url(false, &format!("/~/{key}")))
            .send()
            .await
            .unwrap();

        if !response.status().is_success() {
            todo!();
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok())
            .map(String::from);

        let body = response.bytes().await.unwrap();

        match content_type.as_deref() {
            Some(CONTENT_TYPE_SIGNAL) => Ok(Entry::Signal),
            Some(CONTENT_TYPE_BOOLEAN) => {
                let value = body[0] != 0;
                Ok(Entry::Boolean(value))
            }
            Some(_) => todo!(),
            None => todo!(),
        }
    }
}
