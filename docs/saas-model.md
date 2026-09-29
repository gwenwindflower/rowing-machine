# SaaS accounts, revenue, and usage

`--scenario saas` produces marketing, sales pipeline, account membership, recurring revenue, and product usage data. The `plain` theme supplies business names and labels. The [output schema](output-schema.md#campaigns) lists every column.

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

Leads that do not progress have status `qualified`. Among progressing leads, demo probability is 10%, 50%, or 90% for small, medium, or large employee bands. Demo requests enter the sales pipeline when a rep has capacity; otherwise they remain `demo_requested` without an account. The remaining progressing leads become `converted`, link to an account, and start its self-serve trial the following day. A trial that would begin outside the run leaves the lead `qualified` without an account.

Stage zero writes marketing rows and observes touches; its completion fixes sales assignments. Stage one writes plans, reps, leads, accounts, opportunities, stage entries, and activities. Observation records account arrivals and won close dates. Its completion runs a count-only lifecycle pass, placing user names after lead and rep names. Stage two generates each account independently in account index order. Each lifecycle uses named PCG streams derived from the seed and account index; it retains only one account's dynamic rows at a time. The count pass shares the simulation logic with row generation.

`--target-rows` calibrates on accounts produced by this funnel, with the same 5% or nearest-whole-day rule as ecommerce orders. Calibration searches the available calendar; an unreachable target reports an error suggesting a smaller target or an earlier start date.

## Attribution and paid CAC

These examples use DuckDB SQL after loading the files as tables named for their entities. Compare first-touch and last-touch channels for identified leads:

```sql
select
    first_touch.channel as first_touch_channel,
    last_touch.channel as last_touch_channel,
    count(*) as leads,
    count(l.account_id) as linked_accounts
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

## Sales pipeline

Sales prospects receive an account and opportunity the day after lead creation. Reps own one employee-band segment, with annual costs of 9,000,000, 12,000,000, or 16,000,000 cents for small, medium, or large accounts. Each group of up to 100 scale units supplies one rep per segment. Runs longer than 640 days hire replacements on day 640; runs longer than 730 days end the initial reps' employment on day 730.

Rep capacity ramps over the first 90 days toward 20, 30, or 40 concurrent opportunities by segment. Assignment chooses the least-loaded eligible rep and reserves headroom at capacity. A rep must remain employed through the planned close. Unassigned requests have no opportunity rows.

Opportunities progress through discovery, demo, proposal, negotiation, and a won or lost close. Base cycles span 21–41, 45–74, or 90–134 days by segment. Deals within 30 days of quarter end have a 70% chance of delaying toward its final two weeks. Each admitted deal has a 38% win probability. Activities occur every 3–7 days while the deal is open, including its close day when scheduled.

An opportunity's amount is quoted annual recurring revenue: twelve times the account's initial MRR. An observed win converts its lead and starts a paid subscription on the close date, bypassing trial conversion. Open and lost opportunities retain `demo_requested` leads and have no paid subscription. Stage and activity rows stop at the exclusive run boundary; closes beyond it remain null.

Pipeline at the run boundary uses each opportunity's latest observed stage:

```sql
select stage, count(*) as opportunities, sum(amount) as pipeline_arr_cents
from opportunities
where closed_at is null
group by 1
order by 1;
```

Win rate uses closed deals only, grouped by close month and rep segment:

```sql
select
    date_trunc('month', o.closed_at) as month,
    r.segment,
    count(*) as closed_deals,
    count(*) filter (where o.outcome = 'won') / count(*)::double as win_rate
from opportunities as o
join sales_reps as r on r.id = o.owner_id
where o.closed_at is not null
group by 1, 2
order by 1, 2;
```

## Blended CAC and payback

Blended CAC includes ad spend and rep compensation across self-serve and sales acquisitions. Compensation accrues on employed days at `annual_cost / 365`; this example clips employment to the run and each month. Replace the run bounds with the actual exclusive interval. Acquisitions count each account once, using its first subscription and initial MRR. Seat changes and reactivations do not add acquisitions. Payback is acquisition cost divided by initial MRR, in months, before gross-margin adjustment; it is not a cash-recovery forecast. Months without acquisitions have null CAC and payback.

```sql
with bounds as (
    select date '2023-01-01' as run_start, date '2026-12-31' as run_end
), months as (
    select cast(m as date) as month, b.run_start, b.run_end
    from bounds as b,
        generate_series(date_trunc('month', b.run_start),
            b.run_end - interval '1 day', interval '1 month') as series(m)
), compensation as (
    select
        m.month,
        coalesce(sum(
            greatest(0, date_diff('day',
                greatest(m.month, m.run_start, cast(r.hired_at as date)),
                least(cast(m.month + interval '1 month' as date), m.run_end,
                    coalesce(cast(r.departed_at as date), m.run_end))))
            * r.annual_cost / 365.0
        ), 0) as rep_cost_cents
    from months as m
    left join sales_reps as r on true
    group by 1
), spend as (
    select date_trunc('month', cast(date as date)) as month,
        sum(spend) as spend_cents
    from ad_spend
    group by 1
), first_paid as (
    select account_id, started_at, mrr
    from subscriptions
    qualify row_number() over (partition by account_id order by started_at, id) = 1
), acquisitions as (
    select date_trunc('month', started_at) as month,
        count(*) as paying_accounts, sum(mrr) as initial_mrr_cents
    from first_paid
    group by 1
), monthly as (
    select c.month,
        c.rep_cost_cents + coalesce(s.spend_cents, 0) as acquisition_cost_cents,
        coalesce(a.paying_accounts, 0) as paying_accounts,
        coalesce(a.initial_mrr_cents, 0) as initial_mrr_cents
    from compensation as c
    left join spend as s using (month)
    left join acquisitions as a using (month)
)
select *,
    acquisition_cost_cents / nullif(paying_accounts, 0) as blended_cac_cents,
    acquisition_cost_cents / nullif(initial_mrr_cents, 0) as payback_months
from monthly
order by month;
```

## Trials, membership, and churn

Employee bands begin with 2, 5, or 12 users. Larger bands favor higher plan tiers and have higher user-addition rates. Each account has a fixed latent engagement score; users activate probabilistically from that score within seven days of creation. A 14-day trial converts with probability `0.15 + 0.75 × activated share`. Accounts still in trial or those that did not convert have users but no paid subscription.

Paid accounts add users or lose active members. Billable seats equal active membership, so additions and departures cause expansion and contraction. The user file retains historical users; it does not expose departures. An account emits at most 60 users, including departed users.

Voluntary churn hazard falls exponentially with tenure and rises as engagement falls. Tenure starts at trial arrival for self-serve accounts and at the won close for sales accounts. This yields steeper early cohort loss and flatter mature retention. Churned accounts can reactivate after 45 days if no invoice is beyond its grace period. Latent engagement is internal simulation state, not an extra output column.

## Product usage

Regular sessions start between 09:00 and 16:59 in the account's regional time and last 5–55 minutes, capped at the run boundary. Regional indices use fixed UTC offsets of −5, +1, +9, and −3 hours; daylight saving time is not modeled. Sessions use desktop devices 85% of the time and mobile devices otherwise.

Daily session frequency combines account engagement with a fixed personal multiplier from 0.7 to 1.3. The onboarding multiplier is `1 + exp(-days_since_user_creation / 14)`. Weekend frequency is 8% of weekday frequency, and December 24 through January 1 applies a further 20% multiplier. Frequency fades linearly during the 28 days before a churn event. Regular sessions stop at trial nonconversion or churn, resume on reactivation, and stop permanently at user departure.

Every non-null `users.activated_at` has exactly one activation event at the same midnight UTC timestamp, inside a dedicated one-minute desktop session. These activation markers are exempt from regular session scheduling and business hours. Other sessions contain 4–12 events ordered within `[started_at, ended_at)`.

Events draw from the first 16 generated feature names in the theme. Feature weights depend on plan tier and user role. `event_name` is `<feature>:used` for regular activity and `<feature>:activated` for activation, which uses feature index zero. Themes change labels without changing IDs, timestamps, row counts, or numeric behavior.

## Weekly engagement

This DuckDB query counts regular activity by UTC week, excluding activation markers. It reports only weeks with activity; add a calendar table when zero-activity weeks matter.

```sql
select
    date_trunc('week', occurred_at) as week,
    count(distinct account_id) as active_accounts,
    count(distinct user_id) as active_users,
    count(distinct session_id) as sessions,
    count(*) as feature_uses
from events
where event_name = feature || ':used'
group by 1
order by 1;
```

Compare feature adoption across roles without depending on a theme's specific vocabulary:

```sql
select
    u.role,
    e.feature,
    count(distinct e.user_id) as users,
    count(*) as feature_uses
from events as e
join users as u on u.id = e.user_id
where e.event_name = e.feature || ':used'
group by 1, 2
order by 1, 3 desc, 2;
```

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

This query measures the fraction of self-serve signups with a paid subscription at each monthly age. It excludes sales prospects, includes trial nonconversion in the denominator, and excludes observations past the dataset boundary. Revenue cohorts instead begin at each account's first paid subscription, include sales wins, and exclude nonpaying accounts. Set the observation cutoff to the exclusive end of the run; the default four-year run starts January 1, 2023 and ends December 31, 2026.

```sql
with observations as (
    select
        a.id as account_id,
        date_trunc('month', a.created_at) as cohort,
        age.month_age,
        a.created_at + age.month_age * interval '1 month' as observed_at
    from accounts as a
    cross join generate_series(1, 24) as age(month_age)
    where not exists (
        select 1 from opportunities as o where o.account_id = a.id
    )
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
