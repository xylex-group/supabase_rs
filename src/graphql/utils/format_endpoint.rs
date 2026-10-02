use crate::SupabaseClient;

pub fn endpoint(client: &SupabaseClient) -> String {
    client.service_endpoint_with_segments(&["graphql", "v1"])
}
