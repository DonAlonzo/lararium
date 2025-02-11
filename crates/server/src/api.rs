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

    async fn handle_registry_read(
        &self,
        request: GetRequest,
    ) -> Result<GetResponse> {
        match self.registry.read(&request.topic) {
            Err(lararium_registry::Error::EntryNotFound) => Err(Error::NotFound),
            Err(_) => Err(Error::Unknown),
            Ok(entry) => Ok(GetResponse { entry }),
        }
    }
}
