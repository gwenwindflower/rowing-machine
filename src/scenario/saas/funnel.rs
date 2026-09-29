use crate::engine::stream::Stream;
use crate::output::{Column, ColumnType, EntitySchema};

use super::marketing::Visitor;

pub(super) fn schema() -> EntitySchema {
    use ColumnType::{Text, Timestamp, Uuid};
    EntitySchema {
        name: "leads",
        columns: [
            ("id", Uuid, false),
            ("visitor_id", Uuid, false),
            ("name", Text, false),
            ("email", Text, false),
            ("created_at", Timestamp, false),
            ("lead_source", Text, false),
            ("first_touch_id", Uuid, false),
            ("last_touch_id", Uuid, false),
            ("status", Text, false),
            ("account_id", Uuid, true),
        ]
        .into_iter()
        .map(|(name, column_type, nullable)| Column {
            name,
            column_type,
            nullable,
        })
        .collect(),
        primary_key: vec!["id"],
    }
}

pub(super) fn requests_demo(seed: u64, visitor: usize, band: usize) -> bool {
    Stream::derive(seed, "saas.funnel.route", &[visitor as u64]).uniform() < [0.1, 0.5, 0.9][band]
}

pub(super) fn status(seed: u64, visitor: &Visitor) -> Option<&'static str> {
    let mut rng = Stream::derive(seed, "saas.funnel", &[visitor.index as u64]);
    if rng.uniform() >= visitor.lead_rate {
        return None;
    }
    if rng.uniform() >= visitor.quality {
        return Some("qualified");
    }
    let band = Stream::derive(seed, "saas.profile", &[visitor.index as u64]).index(3);
    Some(if requests_demo(seed, visitor.index, band) {
        "demo_requested"
    } else {
        "converted"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_leads_mostly_start_trials_and_large_leads_mostly_request_demos() {
        let demos: Vec<_> = (0..3)
            .map(|band| {
                (0..1000)
                    .filter(|&index| requests_demo(42, index, band))
                    .count()
            })
            .collect();
        assert!(demos[0] < 200, "{demos:?}");
        assert!((400..600).contains(&demos[1]), "{demos:?}");
        assert!(demos[2] > 800, "{demos:?}");
    }
}
