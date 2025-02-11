impl api::Handler for crate::Server {
    async fn handle_join(
        &self,
        request: api::JoinRequest,
    ) -> Result<api::JoinResponse, api::Error> {
        tracing::debug!("Client joined");
        let Ok(certificate) = self.identity.sign_csr(&request.csr, "random-name") else {
            todo!();
        };
        Ok(api::JoinResponse {
            certificate,
            ca: self.ca.clone(),
        })
    }
}
