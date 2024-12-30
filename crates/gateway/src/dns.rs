use crate::prelude::*;
use lararium_dns::*;

impl Handler for crate::Gateway {
    async fn handle_dns_query(
        &self,
        query: &Query,
    ) -> Option<Response> {
        debug!("DNS query: {query:?}");
        Some(Response {
            transaction_id: query.transaction_id,
            operation_code: OperationCode::StandardQuery,
            authoritative: false,
            recursion_desired: query.recursion_desired,
            recursion_available: false,
            response_code: ResponseCode::NoError,
            answers: vec![Answer {
                name: "lararium.gateway".into(),
                record_type: RecordType::A,
                class: Class::Internet,
                ttl: 300,
                data: vec![127, 0, 0, 1],
            }],
        })
    }
}
