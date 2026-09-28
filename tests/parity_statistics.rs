mod support;

use std::collections::BTreeMap;
use std::fs;

#[test]
#[allow(clippy::float_cmp)]
fn output_statistics_join_years_stores_and_personas() {
    let directory = tempfile::tempdir().unwrap();
    let files = [
        (
            "stores",
            "id,name,opened_at,tax_rate\ns1,Hall,2023-01-01T00:00:00,0.1\n",
        ),
        (
            "customers",
            "id,name,guild_rank\nc1,A,initiate\nc2,B,master\n",
        ),
        (
            "products",
            "sku,name,type,power_level,price,description\np1,P,elixir,common,100,D\n",
        ),
        (
            "supplies",
            "id,name,cost,volatile,origin_region,sku\ns1,S,10,False,Hall,p1\n",
        ),
        (
            "orders",
            "id,customer,ordered_at,store_id,subtotal,tax_paid,order_total\no1,c1,2023-12-31T23:55:00,s1,100,10,110\no2,c2,2024-01-01T09:00:00,s1,200,20,220\no3,c1,2024-01-02T09:00:00,s1,100,10,110\n",
        ),
        (
            "items",
            "id,order_id,sku\ni1,o1,p1\ni2,o2,p1\ni3,o2,p1\ni4,o3,p1\n",
        ),
        (
            "sparrows",
            "id,user_id,sent_at,content\nt1,c1,2024-01-01T00:05:00,Hello\n",
        ),
    ];
    for (entity, content) in files {
        fs::write(directory.path().join(format!("raw_{entity}.csv")), content).unwrap();
    }
    let personas = BTreeMap::from([
        ("c1".to_owned(), "Courier".to_owned()),
        ("c2".to_owned(), "Herbalist".to_owned()),
    ]);
    let stats = support::stats::summarize(directory.path(), "raw", &personas).unwrap();
    assert_eq!(stats["rows.orders.2023"], 1.0);
    assert_eq!(stats["rows.orders.2024"], 2.0);
    assert_eq!(stats["rows.customers.2023"], 1.0);
    assert_eq!(stats["rows.customers.2024"], 1.0);
    assert_eq!(stats["rows.items.2024"], 3.0);
    assert_eq!(stats["rows.sparrows.2024"], 1.0);
    assert_eq!(stats["rows.stores.2023"], 1.0);
    assert_eq!(stats["rows.products.static"], 1.0);
    assert_eq!(stats["rows.supplies.static"], 1.0);
    assert_eq!(stats["persona_order_share.Courier"], 2.0 / 3.0);
    assert_eq!(stats["persona_order_share.Herbalist"], 1.0 / 3.0);
    assert_eq!(stats["mean_items_per_order"], 4.0 / 3.0);
    assert_eq!(stats["mean_order_total.Hall"], 440.0 / 3.0);
    assert_eq!(stats["sparrow_rate"], 1.0 / 3.0);
    assert_eq!(stats["guild_rank.initiate"], 1.0);
    assert_eq!(stats["guild_rank.master"], 1.0);
}

#[test]
#[allow(clippy::float_cmp)]
fn empty_output_has_finite_zero_rates() {
    let directory = tempfile::tempdir().unwrap();
    let stats = support::stats::summarize(directory.path(), "raw", &BTreeMap::new()).unwrap();
    assert_eq!(stats["mean_items_per_order"], 0.0);
    assert_eq!(stats["sparrow_rate"], 0.0);
    assert!(stats.values().all(|value| value.is_finite()));
}

#[test]
fn items_without_an_order_cannot_be_assigned_a_year() {
    let directory = tempfile::tempdir().unwrap();
    fs::write(
        directory.path().join("raw_items.csv"),
        "id,order_id,sku\ni1,missing,p1\n",
    )
    .unwrap();
    let error = support::stats::summarize(directory.path(), "raw", &BTreeMap::new()).unwrap_err();
    assert!(error.to_string().contains("unknown order missing"));
}
