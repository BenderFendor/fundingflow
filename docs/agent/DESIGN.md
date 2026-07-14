# FundingFlow Design Rules

This file is the source of truth for FundingFlow frontend design rules. Update it when the site design principles change.

## Direction

FundingFlow uses an archival intelligence terminal style. The interface should feel like public records, financial ledgers, field reports, and data screens in one system.

The visual language combines:

- Archival paper with hard black rules, registration marks, and oversized index numerals.
- Dark evidence terminals with dense source labels and high-contrast values.
- Blueprint panels for methodology, provenance, and system explanations.
- A tight accent set: acid green, yellow, orange, cyan, lavender, and electric blue.
- Ranked ledgers, block charts, and evidence matrices instead of generic card piles.

## Core Tokens

- Background: near-black with a restrained grid, grain, and occasional radial signal glow.
- Paper: warm off-white for primary search surfaces, document introductions, and source-backed explanations.
- Terminal: raised near-black panels for operational data and results.
- Electric blue: provenance, methodology, and system-detail panels.
- Acid green: active search, system status, links, and high-signal labels.
- Yellow: primary public-money totals and selected derived indicators.
- Orange: pressure, warnings, and high-attention derived metrics.
- Lavender: ratios, comparisons, and secondary derived metrics.
- Cyan: calm summaries and national price context.

Keep the palette tight. Do not introduce purple gradients, soft SaaS backgrounds, glass-card piles, or unrelated accent colors.

## Typography

- Use Space Grotesk or the system sans fallback for display and interface copy.
- Use Azeret Mono or the system monospace fallback for money, percentages, counts, dates, source IDs, and controls.
- Use `ff-mono-num` or `ff-pixel` for tabular values.
- Use `ff-micro` and `ff-kicker` for source tags, scopes, section labels, and system status.
- Large headings should use tight tracking and compact line-height.
- Do not scale important text with viewport units.

## Layout

- Start product screens with the working surface: search, metrics, ledgers, evidence, or methodology.
- Use square or lightly rounded panels. Most product panels should not look like consumer SaaS cards.
- Keep primary content inside a shared maximum width of about 1440 pixels.
- Use thin continuous rules to connect related information across a grid.
- Use strong contrast between paper, terminal, and blueprint surfaces.
- Large empty panels are not acceptable. Missing data must name the missing source or import and provide a useful adjacent action.
- Keep dense pages readable on mobile by collapsing grids into ordered ledgers, not by shrinking text.

## Interaction

- Every interactive element must have a visible `:focus-visible` state.
- Links that navigate to records should use semantic links rather than clickable cards implemented as buttons.
- Hover motion should be restrained to small vertical shifts or color changes.
- Respect reduced-motion preferences.
- Search must expose loading, error, empty, and result states to assistive technology.
- Sticky navigation should remain compact and preserve the working surface.

## Graphs And Data Display

- A graph or chart should answer a concrete question before the user reads the rows.
- Use block bars, ranked ledgers, and compact matrix tables for first-pass scanning.
- Show source, date, vintage, and geography where the value can be misread.
- Do not display decorative charts that are disconnected from actual observations.
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
- Use the inspiration folder as direction, not as a literal asset dump unless the user asks to display images.
- Prefer shared primitives in `src/components/ui.tsx`, `site-header.tsx`, and `site-footer.tsx` over repeated page-local markup.
- After UI changes, run typecheck, lint, and build when feasible.
- Render or screenshot desktop and mobile views when browser tooling is available.
- When design principles change, update this file in the same change.
