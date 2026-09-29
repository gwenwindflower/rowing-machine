use anyhow::{Context, Result};
use jiff::civil::Date;

use crate::engine::stream::Stream;
use crate::output::{Column, ColumnType, EntitySchema, Value};
use crate::scenario::UnitRows;
use crate::theme::Theme;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Visitor {
    pub index: usize,
    pub id: [u8; 16],
    pub first_touch_id: [u8; 16],
    pub last_touch_id: [u8; 16],
    pub channel: &'static str,
    pub lead_rate: f64,
    pub quality: f64,
}

pub(super) struct MarketingDay {
    pub rows: UnitRows,
    #[cfg(test)]
    pub visitors: Vec<Visitor>,
}

const CHANNELS: [&str; 6] = [
    "paid_search",
    "paid_social",
    "display",
    "content",
    "referral",
    "direct",
];
const WEIGHTS: [usize; 6] = [25, 20, 10, 20, 10, 15];
const CPC: [i64; 3] = [600, 250, 100];

pub(super) fn schemas() -> Vec<EntitySchema> {
    use ColumnType::{Cents, Date, Integer, Text, Timestamp, Uuid};
    [
        (
            "campaigns",
            vec![
                ("id", Uuid, false),
                ("channel", Text, false),
                ("name", Text, false),
                ("started_at", Timestamp, false),
                ("ended_at", Timestamp, true),
                ("daily_budget", Cents, false),
            ],
            vec!["id"],
        ),
        (
            "ad_spend",
            vec![
                ("date", Date, false),
                ("campaign_id", Uuid, false),
                ("impressions", Integer, false),
                ("clicks", Integer, false),
                ("spend", Cents, false),
            ],
            vec!["date", "campaign_id"],
        ),
        (
            "touches",
            vec![
                ("id", Uuid, false),
                ("visitor_id", Uuid, false),
                ("campaign_id", Uuid, true),
                ("channel", Text, false),
                ("occurred_at", Timestamp, false),
                ("landing_page", Text, false),
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

pub(super) fn daily_capacity(scale: usize) -> Result<usize> {
    scale
        .checked_mul(3)
        .and_then(|value| value.checked_add(6))
        .context("--scale exceeds the marketing population limit; use a smaller value")
}

fn base_count(scale: usize, channel: usize) -> usize {
    (scale / 100 * WEIGHTS[channel] + (scale % 100 * WEIGHTS[channel]).div_ceil(100)).max(1)
}

fn counts(seed: u64, scale: usize, day: usize) -> Result<[usize; 6]> {
    daily_capacity(scale)?;
    let mut counts = [0; 6];
    for (channel, count) in counts.iter_mut().enumerate() {
        let base = base_count(scale, channel);
        *count = if channel < 3 {
            let change =
                Stream::derive(seed, "saas.marketing.spend", &[day as u64, channel as u64])
                    .index(21);
            base + base / 100 * change + base % 100 * change / 100 - base / 10
        } else {
            base + usize::try_from(base as u128 * day as u128 / (day as u128 + 1460))?
        };
    }
    Ok(counts)
}

pub(super) fn visitors(seed: u64, scale: usize, day: usize) -> Result<Vec<Visitor>> {
    let offset = day
        .checked_mul(daily_capacity(scale)?)
        .context("--days exceeds the marketing population limit; use fewer days")?;
    let mut visitors = Vec::new();
    for (channel, count) in counts(seed, scale, day)?.into_iter().enumerate() {
        for _ in 0..count {
            let index = offset
                .checked_add(visitors.len())
                .context("marketing visitor index overflow")?;
            let first_touch_id = Stream::derive(seed, "saas.touch", &[index as u64, 0]).uuid();
            let last_touch_id =
                if Stream::derive(seed, "saas.marketing.return", &[index as u64]).uniform() < 0.4 {
                    Stream::derive(seed, "saas.touch", &[index as u64, 1]).uuid()
                } else {
                    first_touch_id
                };
            visitors.push(Visitor {
                index,
                id: Stream::derive(seed, "saas.visitor", &[index as u64]).uuid(),
                first_touch_id,
                last_touch_id,
                channel: CHANNELS[channel],
                lead_rate: [0.08, 0.04, 0.02, 0.07, 0.10, 0.05][channel],
                quality: [0.40, 0.20, 0.12, 0.35, 0.55, 0.30][channel],
            });
        }
    }
    Ok(visitors)
}

pub(super) fn generate(
    seed: u64,
    scale: usize,
    day: usize,
    start_date: Date,
    theme: &Theme,
) -> Result<MarketingDay> {
    let date = start_date.checked_add(jiff::Span::new().days(i64::try_from(day)?))?;
    let timestamp = date
        .at(0, 0, 0, 0)
        .to_zoned(jiff::tz::TimeZone::UTC)?
        .timestamp()
        .as_microsecond();
    let flight = day / 90;
    let start = start_date
        .checked_add(jiff::Span::new().days(i64::try_from(flight * 90)?))?
        .at(0, 0, 0, 0)
        .to_zoned(jiff::tz::TimeZone::UTC)?
        .timestamp()
        .as_microsecond();
    let end = start
        .checked_add(90 * 86_400_000_000)
        .context("campaign end timestamp overflow")?;
    let counts = counts(seed, scale, day)?;
    let mut rows = Vec::new();
    for channel in 0..3 {
        let campaign = Value::Uuid(
            Stream::derive(seed, "saas.campaign", &[channel as u64, flight as u64]).uuid(),
        );
        if day.is_multiple_of(90) {
            rows.push((
                "campaigns",
                vec![
                    campaign.clone(),
                    Value::Text(CHANNELS[channel].into()),
                    Value::Text(theme.name(seed, "campaign", flight * 3 + channel)),
                    Value::Timestamp(start),
                    Value::Timestamp(end),
                    Value::Cents(i64::try_from(base_count(scale, channel))? * CPC[channel]),
                ],
            ));
        }
        let clicks = i64::try_from(counts[channel])?;
        let epoch: Date = "1970-01-01".parse()?;
        rows.push((
            "ad_spend",
            vec![
                Value::Date(epoch.until(date)?.get_days()),
                campaign,
                Value::Integer(clicks * [20, 40, 80][channel]),
                Value::Integer(clicks),
                Value::Cents(clicks * CPC[channel]),
            ],
        ));
    }
    let visitors = visitors(seed, scale, day)?;
    for visitor in &visitors {
        let channel = CHANNELS
            .iter()
            .position(|&channel| channel == visitor.channel)
            .expect("known channel");
        let campaign = if channel < 3 {
            Value::Uuid(
                Stream::derive(seed, "saas.campaign", &[channel as u64, flight as u64]).uuid(),
            )
        } else {
            Value::Null
        };
        rows.push((
            "touches",
            vec![
                Value::Uuid(visitor.first_touch_id),
                Value::Uuid(visitor.id),
                campaign,
                Value::Text(visitor.channel.into()),
                Value::Timestamp(timestamp),
                Value::Text(
                    ["/pricing", "/product", "/", "/learn", "/invite", "/"][channel].into(),
                ),
            ],
        ));
        if visitor.last_touch_id != visitor.first_touch_id {
            rows.push((
                "touches",
                vec![
                    Value::Uuid(visitor.last_touch_id),
                    Value::Uuid(visitor.id),
                    Value::Null,
                    Value::Text(
                        if visitor.channel == "direct" {
                            "content"
                        } else {
                            "direct"
                        }
                        .into(),
                    ),
                    Value::Timestamp(timestamp + 1_000_000),
                    Value::Text("/signup".into()),
                ],
            ));
        }
    }
    Ok(MarketingDay {
        rows,
        #[cfg(test)]
        visitors,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(index: usize) -> MarketingDay {
        generate(
            42,
            100,
            index,
            "2023-01-01".parse().unwrap(),
            &Theme::load("plain").unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn paid_clicks_equal_touches_and_spend_tracks_budget() {
        let output = day(0);
        let campaigns: Vec<_> = output
            .rows
            .iter()
            .filter(|(entity, _)| *entity == "campaigns")
            .collect();
        assert_eq!(campaigns.len(), 3);
        for (_, campaign) in campaigns {
            let spend: Vec<_> = output
                .rows
                .iter()
                .filter(|(entity, row)| *entity == "ad_spend" && row[1] == campaign[0])
                .collect();
            assert_eq!(spend.len(), 1);
            let row = &spend[0].1;
            let (
                Value::Integer(impressions),
                Value::Integer(clicks),
                Value::Cents(actual),
                Value::Cents(budget),
            ) = (&row[2], &row[3], &row[4], &campaign[5])
            else {
                panic!("invalid spend types")
            };
            assert!(*clicks > 0 && clicks <= impressions);
            assert!((actual - budget).abs() <= budget / 4);
            assert_eq!(
                output
                    .rows
                    .iter()
                    .filter(|(entity, touch)| *entity == "touches" && touch[2] == campaign[0])
                    .count(),
                usize::try_from(*clicks).unwrap()
            );
        }
    }

    #[test]
    fn campaigns_end_when_the_following_flight_begins() {
        let first = day(0);
        let next = day(90);
        let first_campaign = &first
            .rows
            .iter()
            .find(|(entity, _)| *entity == "campaigns")
            .unwrap()
            .1;
        let next_campaign = &next
            .rows
            .iter()
            .find(|(entity, _)| *entity == "campaigns")
            .unwrap()
            .1;
        assert_ne!(first_campaign[0], next_campaign[0]);
        assert_eq!(first_campaign[4], next_campaign[3]);
        assert_eq!(
            next.rows
                .iter()
                .filter(|(entity, _)| *entity == "ad_spend")
                .count(),
            3
        );
    }

    #[test]
    fn unpaid_visits_grow_and_multitouch_paths_cross_channels() {
        let early = day(0);
        let late = day(730);
        for channel in ["content", "referral", "direct"] {
            let count = |output: &MarketingDay| {
                output
                    .rows
                    .iter()
                    .filter(|(entity, row)| {
                        *entity == "touches"
                            && row[3] == Value::Text(channel.into())
                            && row[2] == Value::Null
                    })
                    .count()
            };
            assert!(count(&late) > count(&early));
        }
        let multitouch = early
            .visitors
            .iter()
            .filter(|visitor| visitor.first_touch_id != visitor.last_touch_id)
            .count();
        assert!(multitouch * 5 > early.visitors.len());
        for visitor in &early.visitors {
            let first = early
                .rows
                .iter()
                .find(|(entity, row)| {
                    *entity == "touches" && row[0] == Value::Uuid(visitor.first_touch_id)
                })
                .unwrap();
            let last = early
                .rows
                .iter()
                .find(|(entity, row)| {
                    *entity == "touches" && row[0] == Value::Uuid(visitor.last_touch_id)
                })
                .unwrap();
            assert_eq!(first.1[1], Value::Uuid(visitor.id));
            assert_eq!(last.1[1], Value::Uuid(visitor.id));
            if visitor.first_touch_id != visitor.last_touch_id {
                assert_ne!(first.1[3], last.1[3]);
            }
        }
    }
}
