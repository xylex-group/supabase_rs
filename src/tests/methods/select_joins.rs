//! Integration tests for join and nested select support.
//!
//! Tests left join, inner join, many-to-many, and explicit FK join scenarios.
//! Requires: supabase db reset && cargo test select_joins -- --ignored

use crate::query::JoinSpec;
use crate::tests::methods::init::init;
use serde_json::Value;

/// Left join: orchestral_sections with instruments nested; sections with no instruments return empty array.
pub async fn select_joins_left() -> Result<(), String> {
    let client = init().await.map_err(|e| format!("{:?}", e))?;
    let rows: Vec<Value> = client
        .from("orchestral_sections")
        .select_with_joins(
            &["id", "name"],
            &[JoinSpec::new("instruments", &["id", "name"])],
        )
        .execute()
        .await?;

    // Should have sections; woodwinds has instruments, empty section has none
    assert!(!rows.is_empty(), "Expected at least one section");
    let has_instruments = rows.iter().any(|r| {
        r.get("instruments")
            .and_then(|i| i.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    });
    let has_empty = rows.iter().any(|r| {
        r.get("instruments")
            .and_then(|i| i.as_array())
            .map(|a| a.is_empty())
            .unwrap_or(false)
    });
    assert!(
        has_instruments,
        "Expected at least one section with instruments"
    );
    assert!(
        has_empty,
        "Expected at least one section with no instruments (left join)"
    );
    Ok(())
}

/// Inner join: filter to woodwinds only by requiring instruments with name=flute.
pub async fn select_joins_inner() -> Result<(), String> {
    let client = init().await.map_err(|e| format!("{:?}", e))?;
    let rows: Vec<Value> = client
        .from("orchestral_sections")
        .select_with_joins(
            &["id", "name"],
            &[JoinSpec::new("instruments", &["id", "name"]).inner()],
        )
        .eq("instruments.name", "flute")
        .execute()
        .await?;

    assert_eq!(
        rows.len(),
        1,
        "Inner join with flute filter should return only woodwinds"
    );
    assert_eq!(
        rows[0].get("name").and_then(|v| v.as_str()).unwrap_or(""),
        "woodwinds"
    );
    Ok(())
}

/// Many-to-many: teams -> members (users via members).
pub async fn select_joins_m2m() -> Result<(), String> {
    let client = init().await.map_err(|e| format!("{:?}", e))?;
    let rows: Vec<Value> = client
        .from("teams")
        .select_with_joins(
            &["id", "name"],
            &[JoinSpec::new("members", &["team_id", "user_id"])],
        )
        .execute()
        .await?;

    assert!(!rows.is_empty(), "Expected teams with members");
    let has_members = rows.iter().any(|r| {
        r.get("members")
            .and_then(|m| m.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    });
    assert!(has_members, "Expected at least one team with members");
    Ok(())
}
