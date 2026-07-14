import Link from "next/link";
import { SiteFooter } from "@/components/site-footer";
import { SiteHeader } from "@/components/site-header";
import {
  CropMarks,
  DataBadge,
  Decimals,
  DocumentHero,
  EmptyState,
  MetricTile,
  SectionHeading,
} from "@/components/ui";
import { derivedMetricLabel, formatMetricValue, formatMoney, getNationalPulse } from "@/lib/types";

const foodMetricIds = [
  "food_at_home_cpi_yoy",
  "food_away_from_home_cpi_yoy",
  "egg_price",
  "milk_price",
  "white_bread_price",
  "ground_beef_price",
  "chicken_breast_price",
  "banana_price",
];

const methodology = [
  "One unit of each item: one dozen eggs, one gallon of whole milk, one pound of white pan bread, ground beef, boneless chicken breast, and bananas.",
  "Component prices come from the BLS Average Price Data program for the U.S. city average.",
  "CPI changes use unadjusted CPI-U series for food at home and food away from home.",
  "No regional variation, state grocery estimate, or household consumption weight is applied.",
  "Basket share of weekly wage uses national average hourly earnings at 40 hours per week.",
];

export default async function CostBasketPage() {
  let pulse = null;
  try {
    pulse = await getNationalPulse();
  } catch {
    // Documentation remains available while the API is offline.
  }

  const foodMetrics = (pulse?.latest_metrics ?? []).filter((item) =>
    foodMetricIds.includes(item.observation.metric_id),
  );
  const derivedMetrics = (pulse?.derived_metrics ?? []).filter((metric) =>
    ["food_basket_cost", "food_basket_as_percent_weekly_wage"].includes(metric.metric_id),
  );
  const vintage = pulse?.latest_metrics[0]?.observation.vintage_date ?? "Not available";

  return (
    <div className="min-h-screen text-white">
      <SiteHeader />
      <main id="main-content" className="mx-auto w-full max-w-6xl px-4 pb-16 pt-5 sm:px-6 lg:px-8">
        <DocumentHero
          eyebrow="National food prices"
          title="Cost basket"
          description="A compact national pressure indicator built from BLS U.S. city average prices. It is not a state or local grocery-cost estimate."
          index="03"
        />

        <div className="mt-5 flex flex-wrap gap-2">
          <DataBadge tone="acid">Source: BLS</DataBadge>
          <DataBadge>Scope: U.S. city average</DataBadge>
          <DataBadge>Vintage: {vintage}</DataBadge>
        </div>

        {!pulse ? (
          <section className="mt-8">
            <EmptyState
              title="Cost-basket data is unavailable"
              description="The page can render, but the national pulse endpoint or BLS price import is unavailable. Run the BLS CPI price importer after the API and database are ready."
              action={
                <Link href="/data" className="ff-focus-ring inline-flex bg-[#dfff00] px-4 py-2 font-mono text-[9px] font-black uppercase tracking-wider text-black hover:bg-[#efff68]">
                  Inspect the pipeline
                </Link>
              }
            />
          </section>
        ) : (
          <>
            {derivedMetrics.length > 0 ? (
              <section className="mt-8 grid border border-black/20 md:grid-cols-2">
                {derivedMetrics.map((metric, index) => (
                  <article
                    key={metric.metric_id}
                    className={`relative overflow-hidden border-r border-b border-black/15 p-7 text-black sm:p-9 ${index % 2 === 0 ? "bg-[#f8df1d]" : "bg-[#d0f1ed]"}`}
                  >
                    <CropMarks className="text-black/35" />
                    <DataBadge tone="paper">Derived indicator</DataBadge>
                    <h2 className="ff-sans-wide relative z-10 mt-8 max-w-lg text-xl font-black">
                      {derivedMetricLabel(metric.metric_id)}
                    </h2>
                    <p className="ff-pixel relative z-10 mt-5 text-5xl font-black leading-none tracking-[-0.09em] sm:text-6xl">
                      <Decimals>
                        {metric.metric_id.includes("percent") ? `${metric.value.toFixed(2)}%` : formatMoney(metric.value)}
                      </Decimals>
                    </p>
                    <p className="relative z-10 mt-8 border-t border-black/15 pt-4 font-mono text-[9px] uppercase leading-5 tracking-wider text-black/55">
                      {metric.formula} / vintage {metric.vintage_date}
                    </p>
                  </article>
                ))}
              </section>
            ) : null}

            <section className="ff-terminal mt-8 overflow-hidden">
              <SectionHeading
                eyebrow="Component ledger"
                title="Latest basket prices"
                description="Each tile reports the source observation date, series identifier, and imported scope."
                meta={<DataBadge tone="acid">{foodMetrics.length} observations</DataBadge>}
              />
              {foodMetrics.length === 0 ? (
                <div className="p-6 sm:p-8">
                  <EmptyState
                    title="No component prices are imported"
                    description="Run the BLS CPI price importer to populate the national food component ledger."
                  />
                </div>
              ) : (
                <div className="grid sm:grid-cols-2 lg:grid-cols-4">
                  {foodMetrics.map((item, index) => (
                    <MetricTile
                      key={item.metric.metric_id}
                      label={item.metric.name}
                      value={<Decimals>{formatMetricValue(item.observation.value, item.metric.unit)}</Decimals>}
                      meta={
                        <>
                          {item.observation.date}<br />
                          {item.observation.source_series_id ?? "No series ID"}
                        </>
                      }
                      accent={index < 2 ? "yellow" : index % 3 === 0 ? "cyan" : "white"}
                    />
                  ))}
                </div>
              )}
            </section>
          </>
        )}

        <section className="ff-blueprint mt-8 overflow-hidden">
          <SectionHeading
            eyebrow="Interpretation"
            title="Basket methodology"
            description="This indicator is intentionally narrow and should not be read as a complete household budget."
            meta={<DataBadge tone="blue">National scope</DataBadge>}
          />
          <ol className="divide-y divide-white/15 px-6 py-2 sm:px-8">
            {methodology.map((item, index) => (
              <li key={item} className="grid gap-4 py-5 sm:grid-cols-[3rem_1fr]">
                <span className="ff-pixel text-xl font-black text-white/30">{String(index + 1).padStart(2, "0")}</span>
                <p className="text-sm leading-6 text-white/72">{item}</p>
              </li>
            ))}
          </ol>
        </section>
      </main>
      <SiteFooter />
    </div>
  );
}
