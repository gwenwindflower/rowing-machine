use anyhow::{Context, Result};
use jiff::civil::Date;

use crate::{
    engine::stream::Stream,
    output::{Column, ColumnType, EntitySchema, Value},
    scenario::UnitRows,
    theme::Theme,
};

const DAY: i64 = 86_400_000_000;
const SEGMENTS: [&str; 3] = ["small", "medium", "large"];
const CAPACITY: [usize; 3] = [20, 30, 40];

pub(super) struct Rep {
    band: usize,
    hired: usize,
    departed: Option<usize>,
}

pub(super) struct Deal {
    pub visitor: usize,
    pub created: usize,
    owner: usize,
    close: usize,
    won: bool,
}

pub(super) struct Sales {
    pub reps: Vec<Rep>,
    pub deals: Vec<Deal>,
    open_closes: Vec<Vec<usize>>,
}

impl Sales {
    pub fn new(scale: usize, days: usize) -> Self {
        let mut reps = Vec::new();
        for _ in 0..scale.div_ceil(100).max(1) {
            for band in 0..3 {
                reps.push(Rep {
                    band,
                    hired: 0,
                    departed: (days > 730).then_some(730),
                });
                if days > 640 {
                    reps.push(Rep {
                        band,
                        hired: 640,
                        departed: None,
                    });
                }
            }
        }
        Self {
            open_closes: vec![Vec::new(); reps.len()],
            reps,
            deals: Vec::new(),
        }
    }

    pub fn admit(
        &mut self,
        seed: u64,
        visitor: usize,
        created: usize,
        start: Date,
    ) -> Result<Option<usize>> {
        let band = Stream::derive(seed, "saas.profile", &[visitor as u64]).index(3);
        let mut rng = Stream::derive(seed, "saas.sales.deal", &[visitor as u64]);
        let duration = [21, 45, 90][band] + rng.index([21, 30, 45][band]);
        let mut close = created + duration;
        let date = start.checked_add(jiff::Span::new().days(i64::try_from(close)?))
            .with_context(|| format!("--start-date {start} puts a sales close beyond the supported calendar; choose an earlier --start-date or fewer --years"))?;
        let quarter_month = ((date.month() - 1) / 3 + 1) * 3;
        let quarter_end = Date::new(date.year(), quarter_month, 1)?.last_of_month();
        let remaining = usize::try_from(date.until(quarter_end)?.get_days())?;
        if remaining <= 30 && rng.uniform() < 0.7 {
            close += remaining.saturating_sub(rng.index(14));
        }
        for closes in &mut self.open_closes {
            closes.retain(|&day| day >= created);
        }
        let owner = self
            .reps
            .iter()
            .enumerate()
            .filter_map(|(index, rep)| {
                if rep.band != band
                    || created < rep.hired
                    || rep.departed.is_some_and(|day| close >= day)
                {
                    return None;
                }
                let capacity = (CAPACITY[band] * (created - rep.hired + 30).min(120) / 120).max(1);
                let active = self.open_closes[index].len();
                (active + 1 < capacity).then_some((active, index))
            })
            .min()
            .map(|(_, index)| index);
        let Some(owner) = owner else { return Ok(None) };
        let index = self.deals.len();
        self.open_closes[owner].push(close);
        self.deals.push(Deal {
            visitor,
            created,
            owner,
            close,
            won: rng.uniform() < 0.38,
        });
        Ok(Some(index))
    }

    pub fn roster(
        &self,
        seed: u64,
        start: Date,
        theme: &Theme,
        name_offset: usize,
    ) -> Result<UnitRows> {
        let base = midnight(start)?;
        Ok(self
            .reps
            .iter()
            .enumerate()
            .map(|(index, rep)| {
                (
                    "sales_reps",
                    vec![
                        id(seed, "saas.sales.rep", index),
                        Value::Text(theme.name(seed, "person", name_offset + index)),
                        Value::Text(SEGMENTS[rep.band].into()),
                        timestamp(base, rep.hired),
                        rep.departed.map_or(Value::Null, |day| timestamp(base, day)),
                        Value::Cents([9_000_000, 12_000_000, 16_000_000][rep.band]),
                    ],
                )
            })
            .collect())
    }

    pub fn rows(
        &self,
        seed: u64,
        index: usize,
        start: Date,
        days: usize,
        amount: i64,
    ) -> Result<UnitRows> {
        let deal = &self.deals[index];
        let base = midnight(start)?;
        let opportunity = id(seed, "saas.sales.opportunity", deal.visitor);
        let owner = id(seed, "saas.sales.rep", deal.owner);
        let terminal = if deal.won {
            "closed_won"
        } else {
            "closed_lost"
        };
        let stages = ["discovery", "demo", "proposal", "negotiation", terminal];
        let mut rows = Vec::new();
        let mut stage = stages[0];
        for (position, name) in stages.into_iter().enumerate() {
            let day = deal.created + (deal.close - deal.created) * position / 4;
            if day >= days {
                break;
            }
            stage = name;
            rows.push((
                "opportunity_stages",
                vec![
                    opportunity.clone(),
                    Value::Text(name.into()),
                    timestamp(base, day),
                ],
            ));
        }
        let closed = deal.close < days;
        rows.push((
            "opportunities",
            vec![
                opportunity.clone(),
                id(seed, "saas.account", deal.visitor),
                id(seed, "saas.lead", deal.visitor),
                owner.clone(),
                timestamp(base, deal.created),
                Value::Text(stage.into()),
                Value::Cents(amount),
                if closed {
                    timestamp(base, deal.close)
                } else {
                    Value::Null
                },
                if closed {
                    Value::Text(if deal.won { "won" } else { "lost" }.into())
                } else {
                    Value::Null
                },
            ],
        ));
        let mut rng = Stream::derive(seed, "saas.sales.activities", &[deal.visitor as u64]);
        let mut day = deal.created;
        let mut activity = 0;
        while day <= deal.close && day < days {
            rows.push((
                "sales_activities",
                vec![
                    Value::Uuid(
                        Stream::derive(
                            seed,
                            "saas.sales.activity",
                            &[deal.visitor as u64, activity],
                        )
                        .uuid(),
                    ),
                    owner.clone(),
                    opportunity.clone(),
                    Value::Text(["email", "call", "meeting"][rng.index(3)].into()),
                    timestamp(base, day),
                ],
            ));
            activity += 1;
            day += 3 + rng.index(5);
        }
        Ok(rows)
    }

    pub fn won(&self, index: usize, days: usize) -> bool {
        self.deals[index].won && self.deals[index].close < days
    }
}

fn id(seed: u64, stream: &str, index: usize) -> Value {
    Value::Uuid(Stream::derive(seed, stream, &[index as u64]).uuid())
}

fn midnight(date: Date) -> Result<i64> {
    Ok(date
        .at(0, 0, 0, 0)
        .to_zoned(jiff::tz::TimeZone::UTC)?
        .timestamp()
        .as_microsecond())
}

fn timestamp(base: i64, day: usize) -> Value {
    Value::Timestamp(base + i64::try_from(day).expect("calendar day fits i64") * DAY)
}

pub(super) fn schemas() -> Vec<EntitySchema> {
    use ColumnType::{Cents, Text, Timestamp, Uuid};
    [
        (
            "sales_reps",
            vec![
                ("id", Uuid, false),
                ("name", Text, false),
                ("segment", Text, false),
                ("hired_at", Timestamp, false),
                ("departed_at", Timestamp, true),
                ("annual_cost", Cents, false),
            ],
            vec!["id"],
        ),
        (
            "opportunities",
            vec![
                ("id", Uuid, false),
                ("account_id", Uuid, false),
                ("lead_id", Uuid, false),
                ("owner_id", Uuid, false),
                ("created_at", Timestamp, false),
                ("stage", Text, false),
                ("amount", Cents, false),
                ("closed_at", Timestamp, true),
                ("outcome", Text, true),
            ],
            vec!["id"],
        ),
        (
            "opportunity_stages",
            vec![
                ("opportunity_id", Uuid, false),
                ("stage", Text, false),
                ("entered_at", Timestamp, false),
            ],
            vec!["opportunity_id", "stage"],
        ),
        (
            "sales_activities",
            vec![
                ("id", Uuid, false),
                ("rep_id", Uuid, false),
                ("opportunity_id", Uuid, false),
                ("activity_type", Text, false),
                ("occurred_at", Timestamp, false),
            ],
            vec!["id"],
        ),
    ]
    .into_iter()
    .map(|(name, columns, primary_key)| EntitySchema {
        name,
        columns: columns
            .into_iter()
            .map(|(name, column_type, nullable)| Column {
                name,
                column_type,
                nullable,
            })
            .collect(),
        primary_key,
    })
    .collect()
}
