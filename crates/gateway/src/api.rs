use lararium_api::*;
use lararium_crypto::CertificateSigningRequest;

impl Handler for crate::Gateway {
    async fn handle_join(
        &mut self,
        request: JoinRequest,
    ) -> Result<JoinResponse> {
        tracing::debug!("Client joined");
        let Ok(csr) = CertificateSigningRequest::from_pem(request.csr.as_bytes()) else {
            todo!();
        };
        let Ok(certificate) = self.identity.sign_csr(&csr, "random-name") else {
            todo!();
        };
        let Ok(certificate) = certificate.to_pem() else {
            todo!();
        };
        let Ok(certificate) = String::from_utf8(certificate) else {
            todo!();
        };
        let Ok(ca) = self.ca.to_pem() else {
            todo!();
        };
        let Ok(ca) = String::from_utf8(ca) else {
            todo!();
        };
        Ok(JoinResponse { certificate, ca })
    }
}
