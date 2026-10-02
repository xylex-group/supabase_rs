#[tokio::test]
async fn add_param_deduplicates() {
    let mut q = crate::query::Query::new();
    q.add_param("limit", "10");
    q.add_param("limit", "10");
    // Duplicate should not be added twice
    let built = q.build();
    assert!(built == "limit=10" || built == "limit=10&");
}

#[tokio::test]
async fn build_orders_params_and_filters() {
    let mut q = crate::query::Query::new();
    q.add_param("select", "id,name");
    q.add_param("limit", "10");
    let s = q.build();
    // Order between params is insertion order in implementation; we just assert both keys exist
    assert!(s.contains("select=id%2Cname"));
    assert!(s.contains("limit=10"));
}

#[test]
fn build_encodes_query_keys_and_values() {
    use crate::query::{Filter, Operator};

    let mut query = crate::query::Query::new();
    query.add_param("select", "id,name");
    query.add_filter(Filter {
        column: "name&select".to_owned(),
        operator: Operator::Equals,
        value: "Ada&role=eq.admin".to_owned(),
    });

    assert_eq!(
        query.build(),
        "select=id%2Cname&name%26select.eq=Ada%26role%3Deq.admin"
    );
}

#[tokio::test]
async fn select_with_joins_builds_postgrest_select() {
    use crate::query::JoinSpec;
    use crate::SupabaseClient;

    let client = SupabaseClient::new("https://test.supabase.co", "test-key").expect("client");
    let qb = client.from("orchestral_sections").select_with_joins(
        &["id", "name"],
        &[JoinSpec::new("instruments", &["id", "name"]).inner()],
    );
    let s = qb.query.build();
    assert!(
        s.contains("select=id%2Cname%2Cinstruments%21inner%28id%2Cname%29"),
        "Expected encoded join select, got: {}",
        s
    );
}

#[test]
fn client_builder_uses_explicit_schema_and_rest_prefix() {
    let client = crate::SupabaseClient::builder("https://example.supabase.co", "key")
        .schema("tenant")
        .rest_prefix("api/v2")
        .build()
        .expect("valid client configuration");

    assert_eq!(client.schema, "tenant");
    assert_eq!(
        client.endpoint("users/team members"),
        "https://example.supabase.co/api/v2/users%2Fteam%20members"
    );

    let root_client = crate::SupabaseClient::builder("https://example.supabase.co", "key")
        .rest_prefix("")
        .build()
        .expect("valid root endpoint configuration");
    assert_eq!(
        root_client.endpoint("users"),
        "https://example.supabase.co/users"
    );
}
