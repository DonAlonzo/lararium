use crate::prelude::*;
use lararium_dhcp::*;

impl Handler for crate::Gateway {
    async fn handle_discover(
        &self,
        discover: Discover,
    ) -> Option<Offer> {
        self.core.read().await.handle_discover(discover).await
    }
}

impl Handler for crate::Core {
    async fn handle_discover(
        &self,
        discover: Discover,
    ) -> Option<Offer> {
        debug!("Received DHCP discover. {discover:#?}");
        None
        //Some(Offer {
        //    transaction_id: discover.transaction_id,
        //    your_ip_address: [192, 168, 1, 2],
        //    server_ip_address: [192, 168, 1, 1],
        //    subnet_mask: [255, 255, 255, 0],
        //    router: [192, 168, 1, 1],
        //    dns_servers: vec![[8, 8, 8, 8], [8, 8, 4, 4]],
        //    lease_time: 60,
        //})
    }
}
