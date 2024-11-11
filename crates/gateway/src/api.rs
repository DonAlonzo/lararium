use lararium_api::*;

impl Handler for crate::Gateway {
    async fn handle_join(
        &self,
        request: JoinRequest,
    ) -> Result<JoinResponse> {
        self.core.read().await.handle_join(request).await
    }
}

impl Handler for crate::Core {
    async fn handle_join(
        &self,
        request: JoinRequest,
    ) -> Result<JoinResponse> {
        tracing::debug!("Client joined");
        let Ok(certificate) = self.identity.sign_csr(&request.csr, "random-name") else {
            todo!();
        };
        Ok(JoinResponse {
            certificate,
            ca: self.ca.clone(),
        })
    }
}
