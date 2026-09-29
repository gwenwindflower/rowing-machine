use crate::output::{Column, ColumnType, EntitySchema};

#[allow(clippy::too_many_lines)]
pub(super) fn entities() -> Vec<EntitySchema> {
    use ColumnType::{Cents, Integer, Text, Timestamp, Uuid};
    let mut entities: Vec<_> = [
        (
            "accounts",
            vec![
                ("id", Uuid, false),
                ("name", Text, false),
                ("industry", Text, false),
                ("employee_band", Text, false),
                ("region", Text, false),
                ("created_at", Timestamp, false),
                ("acquisition_channel", Text, false),
                ("first_touch_id", Uuid, true),
            ],
        ),
        (
            "users",
            vec![
                ("id", Uuid, false),
                ("account_id", Uuid, false),
                ("name", Text, false),
                ("email", Text, false),
                ("role", Text, false),
                ("created_at", Timestamp, false),
                ("activated_at", Timestamp, true),
            ],
        ),
        (
            "sessions",
            vec![
                ("id", Uuid, false),
                ("user_id", Uuid, false),
                ("account_id", Uuid, false),
                ("started_at", Timestamp, false),
                ("ended_at", Timestamp, false),
                ("device", Text, false),
            ],
        ),
        (
            "events",
            vec![
                ("id", Uuid, false),
                ("session_id", Uuid, false),
                ("user_id", Uuid, false),
                ("account_id", Uuid, false),
                ("occurred_at", Timestamp, false),
                ("event_name", Text, false),
                ("feature", Text, false),
            ],
        ),
        (
            "plans",
            vec![
                ("id", Uuid, false),
                ("name", Text, false),
                ("tier", Text, false),
                ("seat_price_monthly", Cents, false),
                ("seat_price_annual", Cents, false),
                ("included_seats", Integer, false),
            ],
        ),
        (
            "subscriptions",
            vec![
                ("id", Uuid, false),
                ("account_id", Uuid, false),
                ("plan_id", Uuid, false),
                ("billing_interval", Text, false),
                ("seats", Integer, false),
                ("mrr", Cents, false),
                ("started_at", Timestamp, false),
                ("ended_at", Timestamp, true),
                ("status", Text, false),
            ],
        ),
        (
            "mrr_movements",
            vec![
                ("id", Uuid, false),
                ("account_id", Uuid, false),
                ("subscription_id", Uuid, false),
                ("movement_type", Text, false),
                ("occurred_at", Timestamp, false),
                ("mrr_delta", Cents, false),
                ("mrr_after", Cents, false),
            ],
        ),
        (
            "invoices",
            vec![
                ("id", Uuid, false),
                ("account_id", Uuid, false),
                ("subscription_id", Uuid, false),
                ("issued_at", Timestamp, false),
                ("period_start", Timestamp, false),
                ("period_end", Timestamp, false),
                ("amount", Cents, false),
                ("paid_at", Timestamp, true),
            ],
        ),
    ]
    .into_iter()
    .map(|(name, columns)| EntitySchema {
        name,
        columns: columns
            .into_iter()
            .map(|(name, column_type, nullable)| Column {
                name,
                column_type,
                nullable,
            })
            .collect(),
        primary_key: vec!["id"],
    })
    .collect();
    entities.extend(super::marketing::schemas());
    entities.push(super::funnel::schema());
    entities.extend(super::sales::schemas());
    entities
}
