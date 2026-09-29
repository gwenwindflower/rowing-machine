# SaaS accounts and revenue

`--scenario saas` produces accounts, users, plans, subscriptions, MRR movements, and invoices. The `plain` theme supplies business names and labels. The [output schema](output-schema.md#accounts) lists every column.

```bash
rowing-machine --scenario saas --seed 42 --years 4 --scale 100
rowing-machine --scenario saas --seed 42 --scale 100 --target-rows 1000 --format parquet
```

## Arrivals and generation

The addressable population is `20 × scale`: 2,000 accounts at the default scale. Each account has an indexed arrival day uniformly distributed over the first 1,460 days. Runs emit only arrivals inside their duration. Longer runs continue those accounts' lifecycles without adding another population. Every arrival has `acquisition_channel = 'direct'` and null `first_touch_id`.

Stage zero writes plans and arrivals in day order, then account index order. Observation records emitted arrivals. At the stage boundary, a count-only lifecycle pass fixes contiguous user-name offsets. Stage one generates each arrived account independently, in account index order. Each lifecycle uses named PCG streams derived from the seed and account index; it retains only one account's dynamic rows at a time. The count pass shares the simulation logic with row generation.

`--target-rows` calibrates on accounts, with the same 5% or nearest-whole-day rule as ecommerce orders. A target larger than the addressable population fails before output with a suggestion to increase `--scale`. Product usage, marketing touches, leads, and sales entities belong to their own planned extensions.

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

These examples use DuckDB SQL after loading the files as tables named for their entities. A date means midnight UTC; intervals ending at that timestamp do not contribute.

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
