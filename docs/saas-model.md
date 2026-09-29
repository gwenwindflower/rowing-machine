# SaaS accounts and revenue

`--scenario saas` produces campaigns, daily ad spend, marketing touches, leads, accounts, users, plans, subscriptions, MRR movements, and invoices. The `plain` theme supplies business names and labels. The [output schema](output-schema.md#campaigns) lists every column.

```bash
rowing-machine --scenario saas --seed 42 --years 4 --scale 100
rowing-machine --scenario saas --seed 42 --scale 100 --target-rows 1000 --format parquet
```

## Marketing and account generation

Paid search, paid social, and display campaigns run in consecutive 90-day flights. Each active campaign emits daily spend near its budget, with exactly one paid touch per click. Cost per click is 600, 250, and 100 cents, respectively. Content, referral, and direct visitors arrive without spend; their daily volume grows toward twice its starting level. `--scale` controls visitor volume.

About 40% of visitors have a second touch on the same day, through a different channel. Leads retain both first and last touch IDs. Their source, and the acquisition channel of any resulting account, follows the first touch.

| First-touch channel | Visit-to-lead probability | Lead progression probability |
| --- | --- | --- |
| `paid_search` | 8% | 40% |
| `paid_social` | 4% | 20% |
| `display` | 2% | 12% |
| `content` | 7% | 35% |
| `referral` | 10% | 55% |
| `direct` | 5% | 30% |

Leads that do not progress have status `qualified`. Among progressing leads, demo probability is 10%, 50%, or 90% for small, medium, or large employee bands. Demo requests remain `demo_requested`; sales opportunities are not emitted. The remaining leads become `converted`, link to an account, and start its self-serve trial the following day. A trial that would begin outside the run leaves the lead `qualified` without an account.

Stage zero writes plans and each day's marketing, leads, and trial accounts. Observation records emitted accounts and lead counts. Leads use contiguous person-name indices; at the stage boundary, a count-only lifecycle pass places user names after the lead names. Stage one generates each account independently, in account index order. Each lifecycle uses named PCG streams derived from the seed and account index; it retains only one account's dynamic rows at a time. The count pass shares the simulation logic with row generation.

`--target-rows` calibrates on accounts produced by this funnel, with the same 5% or nearest-whole-day rule as ecommerce orders. Calibration searches the available calendar; an unreachable target reports an error suggesting a smaller target or an earlier start date.

## Attribution and paid CAC

These examples use DuckDB SQL after loading the files as tables named for their entities. Compare first-touch and last-touch channels for identified leads:

```sql
select
    first_touch.channel as first_touch_channel,
    last_touch.channel as last_touch_channel,
    count(*) as leads,
    count(l.account_id) as trial_accounts
from leads as l
join touches as first_touch on first_touch.id = l.first_touch_id
join touches as last_touch on last_touch.id = l.last_touch_id
group by 1, 2
order by 1, 2;
```

Paid CAC divides monthly channel spend by accounts that first became paying customers in that month, attributed to their first touch. A trial signup is not yet a paying customer. A month without paying acquisitions has null CAC; values are cents per account.

```sql
with spend as (
    select
        date_trunc('month', cast(s.date as date)) as month,
        c.channel,
        sum(s.spend) as spend_cents
    from ad_spend as s
    join campaigns as c on c.id = s.campaign_id
    group by 1, 2
), first_payment as (
    select account_id, min(started_at) as first_paid_at
    from subscriptions
    group by 1
), acquisitions as (
    select
        date_trunc('month', p.first_paid_at) as month,
        a.acquisition_channel as channel,
        count(distinct a.id) as paying_accounts
    from first_payment as p
    join accounts as a on a.id = p.account_id
    group by 1, 2
)
select
    s.month,
    s.channel,
    s.spend_cents,
    coalesce(a.paying_accounts, 0) as paying_accounts,
    s.spend_cents / nullif(a.paying_accounts, 0)::double as paid_cac_cents
from spend as s
left join acquisitions as a using (month, channel)
order by 1, 2;
```

## Trials, membership, and churn

Employee bands begin with 2, 5, or 12 users. Larger bands favor higher plan tiers and have higher user-addition rates. Each account has a fixed latent engagement score; users activate probabilistically from that score within seven days of creation. A 14-day trial converts with probability `0.15 + 0.75 × activated share`. Accounts still in trial or those that did not convert have users but no paid subscription.

Paid accounts add users or lose active members. Billable seats equal active membership, so additions and departures cause expansion and contraction. The user file retains historical users; it does not expose departures. An account emits at most 60 users, including departed users.

Voluntary churn hazard falls exponentially with tenure and rises as engagement falls. This yields steeper early cohort loss and flatter mature retention. Churned accounts can reactivate after 45 days if no invoice is beyond its grace period. Latent engagement is internal simulation state, not an extra output column.

## Subscription and invoice intervals

Three plans cost 1,500, 3,000, and 6,000 cents per seat per month, or 15,000, 30,000, and 60,000 cents per year. The annual MRR per seat is 1,250, 2,500, or 5,000 cents. `included_seats` is reference bundle metadata; all `seats` contribute to MRR. The selected plan and billing interval stay fixed for an account.

Subscriptions are immutable paid intervals: `[started_at, ended_at)`. A membership change closes the current row and opens another with its resulting seats and MRR. An open interval has null `ended_at`. The first positive MRR is `new`, changes above zero are `expansion` or `contraction`, a move to zero is `churn`, and a positive return is `reactivation`. Summing deltas through a timestamp equals both the latest `mrr_after` and the active subscription MRR.

Invoices start at the subscription boundary and renew in calendar months or years. Each period ends at its next renewal, the subscription end, or the exclusive simulation boundary, whichever comes first. Amounts are prorated by covered calendar days and rounded with integer arithmetic. January 31 renewals clamp to February's last day; subsequent cycles start there. Invoice periods never cross subscription boundaries.

Most invoices are paid on issue. About 13% are scheduled 8–20 days late and 2% are never paid. An account with an invoice still unpaid after 30 days churns on day 31, including when that invoice belongs to an ended seat interval. Payments beyond the run boundary appear as null. Ended subscriptions are `canceled`; an open subscription is `past_due` while any account invoice remains unpaid and `active` otherwise. No `trialing` subscription rows are emitted because trials have no paid MRR.

## MRR and ARR at a date

A date means midnight UTC; intervals ending at that timestamp do not contribute.

```sql
select
    coalesce(sum(mrr), 0) as mrr_cents,
    coalesce(sum(mrr), 0) * 12 as arr_cents
from subscriptions
where started_at <= timestamp '2025-01-01 00:00:00'
    and (ended_at is null or ended_at > timestamp '2025-01-01 00:00:00');
```

The ledger provides an independent calculation:

```sql
select coalesce(sum(mrr_delta), 0) as mrr_cents
from mrr_movements
where occurred_at <= timestamp '2025-01-01 00:00:00';
```

Monthly movement totals explain changes in revenue:

```sql
select
    date_trunc('month', occurred_at) as month,
    movement_type,
    sum(mrr_delta) as mrr_change_cents
from mrr_movements
group by 1, 2
order by 1, 2;
```

## Signup-cohort retention

This query measures the fraction of all signups with a paid subscription at each monthly age. It includes trial nonconversion in the denominator and excludes observations past the dataset boundary. Set `run_end` to the exclusive end of the run; the default four-year run starts January 1, 2023 and ends December 31, 2026.

```sql
with observations as (
    select
        a.id as account_id,
        date_trunc('month', a.created_at) as cohort,
        age.month_age,
        a.created_at + age.month_age * interval '1 month' as observed_at
    from accounts as a
    cross join generate_series(1, 24) as age(month_age)
), eligible as (
    select *
    from observations
    where observed_at < timestamp '2026-12-31 00:00:00'
)
select
    e.cohort,
    e.month_age,
    count(distinct case when s.id is not null then e.account_id end)
        / count(distinct e.account_id)::double as paid_retention
from eligible as e
left join subscriptions as s
    on s.account_id = e.account_id
    and s.started_at <= e.observed_at
    and (s.ended_at is null or s.ended_at > e.observed_at)
group by 1, 2
order by 1, 2;
```
