#[test]
fn api_contract_snapshot_stable() {
    let contract = serde_json::json!({
        "data": {
            "id": "00000000-0000-0000-0000-000000000001",
            "alias": "owner_alias",
            "display_name": "Owner",
            "is_private": true
        },
        "pagination": null
    });

    insta::assert_json_snapshot!(contract, @r###"
    {
      "data": {
        "alias": "owner_alias",
        "display_name": "Owner",
        "id": "00000000-0000-0000-0000-000000000001",
        "is_private": true
      },
      "pagination": null
    }
    "###);
}
