# FundingFlow Design Rules

This file is the source of truth for FundingFlow frontend design rules. Update it when the site design principles change.

## Direction

FundingFlow uses an archival intelligence terminal style. The interface should feel like public records, financial ledgers, field reports, and data screens in one system.

The current visual references point to:

- Editorial archive pages with oversized serif ghosts, thin rules, pale paper, and heavy black outlines.
- Bitmap and terminal typography for numbers, series IDs, and source labels.
- Dark scanline dashboards with dense evidence panels.
- Muted sage and paper surfaces with sharp orange, yellow, electric blue, cyan, and lavender panels.
- Simple block charts and ranked ledgers instead of decorative chart chrome.

## Core Tokens

- Background: near-black with faint grid and scanline texture.
- Paper: warm off-white for archival headers and source-backed explanations.
- Terminal: deep black panels for operational data.
- Electric blue: use for blueprint or intelligence panels.
- Yellow: use for primary actions and key public-money totals.
- Orange: use for pressure, risk, or high-attention derived metrics.
- Lavender: use for comparison ratios and secondary metrics.
- Cyan: use for calm summary surfaces and highlight bars.

Keep the palette tight. Do not introduce purple gradients, soft SaaS backgrounds, or random accent colors.

## Typography

- Primary UI type can stay system sans.
- Use `ff-mono-num` for monetary values, percentages, counts, and chart values.
- Use `ff-micro` for labels, metric scopes, source tags, and table headers.
- Use serif type only as a large background or editorial marker, not for dense UI copy.
- Use tight tracking on display headings. Do not scale text with viewport units.

## Layout

- Start product screens with the working surface: search, metrics, charts, ledgers, or evidence.
- Use strong panels, ranked rows, and source labels. Avoid generic dashboard card piles.
- Large empty panels are not acceptable. If data is missing, show the missing source, scope, and a useful next action or adjacent dataset.
- Keep rounded corners under 2rem unless a page already uses a hero-scale panel.
- Prefer visible evidence boards over marketing copy.

## Graphs And Data Display

- Graphs should answer a concrete question before the user reads the rows.
- Use block bars, ranked ledgers, and compact matrix tables for first-pass scanning.
- Show source, date, vintage, and geography where the value can be misread.
- Never combine separate demographic cuts as if they were intersectional data.
- Missing race-by-gender, county, or state food data must be labeled as not imported.
- Do not convert missing data to zero.

## Copy

- Use plain utility copy.
- Labels should name the data scope: national, state, county, source, vintage, or imported status.
- Avoid causal claims. Public-record relationships are documented links, not proof of intent or influence.
- No emojis.
- No em dashes.

## Frontend Workflow

- Before frontend design work, read this file and the nearest `AGENTS.md`.
- Use the inspo folder as direction, not as a literal asset dump unless the user asks to display images.
- After UI changes, run typecheck, lint, and build when feasible.
- Use Chrome MCP screenshots for desktop and mobile.
- When design principles change, update this file in the same change.
