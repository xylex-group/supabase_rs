#[tokio::test]
async fn filter_display_works() {
    let f = crate::query::Filter::new(
        "age".to_owned(),
        crate::query::Operator::GreaterThan,
        "30".to_owned(),
    );
    assert_eq!(f.to_string(), "age.gt=30");
}

#[tokio::test]
async fn sort_display_works() {
    let asc = crate::query::Sort {
        column: "name".to_owned(),
        order: crate::query::SortOrder::Ascending,
    };
    let desc = crate::query::Sort {
        column: "name".to_owned(),
        order: crate::query::SortOrder::Descending,
    };
    assert_eq!(asc.to_string(), "name.asc");
    assert_eq!(desc.to_string(), "name.desc");
}
