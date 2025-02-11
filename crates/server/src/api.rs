use lararium_api::*;

impl Handler for crate::Server {
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
