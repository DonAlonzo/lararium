use lararium_dns::*;

impl Handler for crate::Gateway {
    fn handle_dns_query(
        &self,
        query: &Query,
    ) -> Option<Response> {
        Some(Response {
            transaction_id: query.transaction_id,
            flags: ResponseFlags {
                operation_code: OperationCode::StandardQuery,
                authoritative_answer: false,
                recursion_desired: query.flags.recursion_desired,
                recursion_available: false,
                response_code: ResponseCode::NoError,
            },
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
