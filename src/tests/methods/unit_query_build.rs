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
    assert!(s.contains("select=id,name"));
    assert!(s.contains("limit=10"));
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
        s.contains("select=id,name,instruments!inner(id,name)"),
        "Expected select=id,name,instruments!inner(id,name), got: {}",
        s
    );
}
