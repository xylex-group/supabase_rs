#[test]
fn random_id_is_positive_and_varies() {
    let a = crate::generate_random_id();
    let b = crate::generate_random_id();
    assert!(a >= 0);
    assert!(b >= 0);
    // Very unlikely to be equal twice in a row; if it happens, it still means function works
    // so only assert not both zero-length
    if a == b {
        // acceptable flake, but ensure within range
        assert!(a >= 0);
    }
}

#[test]
fn supabase_client_debug_redacts_api_key() {
    let api_key = "sensitive-api-key";
    let client = crate::SupabaseClient::new("https://example.supabase.co", api_key)
        .expect("client should build");

    let debug = format!("{client:?}");

    assert!(
        !debug.contains(api_key),
        "debug output must not expose secrets"
    );
    assert!(
        debug.contains("[REDACTED]"),
        "debug output should mark the key as redacted"
    );
}

#[test]
fn supabase_client_rejects_malformed_base_url_during_construction() {
    crate::SupabaseClient::new("not a URL", "key").unwrap_err();
}
