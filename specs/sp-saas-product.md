# SaaS product and revenue

## Goals

The `saas` scenario simulates a B2B software company selling seat-based subscriptions to other businesses. This spec covers what happens after an account exists: its users, their sessions and product events, its subscriptions, and the revenue those produce. The data should let a learner compute MRR and ARR at any date, break MRR into new, expansion, contraction, churn, and reactivation, build signup and revenue cohort retention curves, and relate product engagement to conversion and churn. How accounts arrive (marketing, leads, sales) lives in `gm-go-to-market.md`.

## Vocabulary

- **Account** — a customer company or workspace; the unit that subscribes and pays.
- **User** — a person inside an account who uses the product.
- **Trial** — a time-limited free period before an account's first paid subscription.
- **MRR** — monthly recurring revenue in cents; annual plans contribute their price divided by 12, rounded to cents.
- **MRR movement** — one change to an account's MRR, typed new, expansion, contraction, churn, or reactivation.
- **Activation** — a user completing the product's key setup events within their first week.

## Requirements

### Entities

- **sp-R001** — `accounts`: `id, name, industry, employee_band, region, created_at, acquisition_channel, first_touch_id`; PK `id`; `first_touch_id` is nullable and references `touches.id`.
- **sp-R002** — `users`: `id, account_id, name, email, role, created_at, activated_at`; PK `id`; FK `account_id → accounts.id`; `activated_at` is nullable.
- **sp-R003** — `plans`: `id, name, tier, seat_price_monthly, seat_price_annual, included_seats`; static; PK `id`.
- **sp-R004** — `subscriptions`: `id, account_id, plan_id, billing_interval, seats, mrr, started_at, ended_at, status`; PK `id`; FKs to `accounts` and `plans`; `ended_at` is nullable.
- **sp-R005** — `mrr_movements`: `id, account_id, subscription_id, movement_type, occurred_at, mrr_delta, mrr_after`; PK `id`; FKs to `accounts` and `subscriptions`.
- **sp-R006** — `invoices`: `id, account_id, subscription_id, issued_at, period_start, period_end, amount, paid_at`; PK `id`; `paid_at` is nullable.
- **sp-R007** — `sessions`: `id, user_id, account_id, started_at, ended_at, device`; PK `id`; FKs to `users` and `accounts`.
- **sp-R008** — `events`: `id, session_id, user_id, account_id, occurred_at, event_name, feature`; PK `id`; FKs to `sessions`, `users`, and `accounts`.

### Revenue integrity

- **sp-R010** — For every account, summing `mrr_delta` over its movements up to any timestamp equals the `mrr_after` of its latest movement at that timestamp.
- **sp-R011** — An account's MRR at any date equals the sum of `mrr` over its subscriptions active on that date.
- **sp-R012** — Movement types follow MRR: `new` is an account's first move above zero, `churn` a move to zero, `reactivation` a move above zero after churn, and `expansion` or `contraction` any other increase or decrease.
- **sp-R013** — A subscription's `mrr` equals its seats times the plan's seat price for its billing interval, normalized to monthly cents.
- **sp-R014** — Invoices tile each subscription's active period with no gaps or overlaps: monthly plans invoice monthly and annual plans invoice yearly, in advance.
- **sp-R015** — Some invoices are paid late and a small share are never paid; an account with unpaid invoices past a grace period churns.
- **sp-R016** — `status` is `trialing`, `active`, `past_due`, or `canceled`, and agrees with `ended_at` and the account's invoices on the run's last day.

### Accounts and users

- **sp-R020** — Larger `employee_band` accounts start with more seats, add users faster, and choose higher tiers more often.
- **sp-R021** — Every user's `created_at` falls on or after their account's `created_at`, and `activated_at`, when present, falls within seven days of `created_at`.
- **sp-R022** — Seat count changes (expansion and contraction) follow user growth and loss inside the account, not independent random draws.
- **sp-R023** — Accounts churn at a rate that falls with tenure and rises with low engagement, so signup-cohort retention curves decline steeply early and flatten.
- **sp-R024** — Trials convert to paid at a rate that rises with the share of trial users who activated.

### Product usage

- **sp-R030** — Sessions follow a work-week rhythm: weekday sessions dominate, cluster in business hours for the account's region, and dip around year-end holidays.
- **sp-R031** — Each session's events fall within its `started_at` and `ended_at`, and every session has at least one event.
- **sp-R032** — Event names come from the theme's feature catalog, and each feature's adoption varies by plan tier and user role.
- **sp-R033** — A user's session frequency decays toward a steady personal rate after onboarding, and falls toward zero in the weeks before their account churns.
- **sp-R034** — `events` is the highest-volume entity, scaling with `--scale` times active users times days.
