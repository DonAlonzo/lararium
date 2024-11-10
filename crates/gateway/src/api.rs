use lararium_api::*;
use lararium_crypto::CertificateSigningRequest;

impl Handler for crate::Gateway {
    async fn handle_join(
        &mut self,
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
