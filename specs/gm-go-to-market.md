# SaaS go-to-market

## Goals

How the `saas` scenario's accounts arrive: paid and organic marketing produce anonymous visits, visits become leads, and leads become accounts through a self-serve trial or a sales-led opportunity. The data should let a learner build the full funnel by channel and campaign, compute paid and blended CAC, CAC payback, and LTV:CAC, compare first-touch and last-touch attribution, and measure sales cycle length, win rate, and rep productivity. Revenue after the sale lives in `sp-saas-product.md`, and every entity's columns live in `docs/output-schema.md`.

## Vocabulary

- **Funnel** — the path from anonymous visit to lead, opportunity or trial, and paying account.
- **Channel** — a marketing source: paid search, paid social, display, content, events, referral, direct.
- **Campaign** — a time-bounded paid or organic effort within one channel.
- **Touch** — one marketing interaction by an anonymous visitor, optionally tied to a campaign.
- **Lead** — a visitor who identified themselves through a signup, trial, or demo request.
- **Opportunity** — a sales-owned deal for one account, moving through pipeline stages to won or lost.
- **CAC** — customer acquisition cost: marketing spend (paid CAC), or marketing spend plus sales cost (blended CAC), divided by new paying accounts in a period.

## Requirements

### Marketing

- **gm-R010** — Every active paid campaign has one `ad_spend` row per day, with `clicks ≤ impressions` and `spend` near its daily budget.
- **gm-R011** — Paid touches per campaign per day follow that day's clicks, so spend, clicks, and touches rise and fall together.
- **gm-R012** — Channels differ in cost per click, visit-to-lead rate, and lead quality, so their CAC and funnel shapes are visibly distinct.
- **gm-R013** — Organic, referral, and direct touches grow steadily over the timeline without spend.
- **gm-R014** — A visitor can have many touches before becoming a lead, so first-touch and last-touch attribution disagree for a meaningful share of leads.

### Funnel

- **gm-R020** — Small-band leads mostly start self-serve trials, and large-band leads mostly request demos that become opportunities.
- **gm-R021** — Each funnel stage converts a fraction of the previous one, so counts from touch to lead to trial or opportunity to paid account never increase.
- **gm-R022** — Every paying account traces to exactly one lead, and every converted lead has an `account_id`.
- **gm-R023** — An account's `acquisition_channel` and `first_touch_id` match its lead's first touch, or `direct` with a null touch when the lead had no touches.

### Sales

- **gm-R030** — Opportunities move through `discovery`, `demo`, `proposal`, `negotiation`, then `closed_won` or `closed_lost`, recording each stage entered in `opportunity_stages`, in order and without skipping.
- **gm-R031** — Sales cycles lengthen and deal amounts grow with the account's employee band.
- **gm-R032** — Opportunities close more often in the last two weeks of each calendar quarter.
- **gm-R033** — A won opportunity's `closed_at` equals its account's first paid subscription's `started_at`, and its `amount` equals that subscription's `mrr` times 12.
- **gm-R034** — Every opportunity's owner was employed on its `created_at`, and every sales activity falls between its opportunity's `created_at` and `closed_at`.
- **gm-R035** — Reps ramp to full productivity over their first quarter, and each rep's open opportunities stay under a segment capacity.

### Metrics

- **gm-R040** — Paid CAC per channel and month is computable from `ad_spend` and first-touch account attribution alone.
- **gm-R041** — Blended CAC is computable by adding each rep's `annual_cost`, prorated by the days they were employed in the period, to marketing spend.
- **gm-R042** — Across the default run, blended CAC payback computed from new-account MRR falls between 6 and 36 months.

Retired: gm-R001–gm-R008.
