import Link from "next/link";
import { getNationalPulse, formatMetricValue, derivedMetricLabel, formatMoney } from "@/lib/types";

export default async function CostBasketPage() {
  let pulse = null;
  try {
    pulse = await getNationalPulse();
  } catch {
    // API not running
  }

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

  const foodMetrics = (pulse?.latest_metrics ?? []).filter((item) =>
    foodMetricIds.includes(item.observation.metric_id)
  );

  const derivedMetrics = (pulse?.derived_metrics ?? []).filter((d) =>
    ["food_basket_cost", "food_basket_as_percent_weekly_wage"].includes(d.metric_id)
  );

  return (
    <main className="mx-auto min-h-screen max-w-5xl px-4 py-8 text-slate-100">
      <header className="ff-paper ff-heavy-border mb-8 rounded-[1.75rem] p-6 text-black sm:p-8">
        <Link href="/" className="ff-focus-ring ff-micro text-sm font-black text-black/45 hover:text-black">
          &larr; Back to search
        </Link>
        <p className="ff-micro mt-8 text-sm font-black text-black/50">National Food Prices</p>
        <h1 className="mt-2 text-6xl font-black leading-[0.9] tracking-[-0.08em]">Cost Basket</h1>
        <p className="mt-4 max-w-2xl text-lg font-semibold text-black/65">
          BLS CPI-U U.S. city average prices for a representative food basket.
          All prices are national averages; they do not reflect state or local grocery prices.
        </p>
        <p className="mt-2 text-xs font-semibold text-black/45">
          Source: Bureau of Labor Statistics API. Scope: U.S. city average. Vintage: {pulse?.latest_metrics[0]?.observation.vintage_date ?? "N/A"}.
        </p>
      </header>

      {!pulse ? (
        <div className="ff-terminal rounded-xl border border-dashed border-white/10 p-10 text-center text-sm text-slate-500">
          Cost basket data is unavailable. Start the API server and run <code className="bg-white/10 px-2 py-0.5 rounded">cargo run --bin cli -- import --source bls-cpi-prices</code> to populate live prices.
        </div>
      ) : (
        <>
          {derivedMetrics.length > 0 && (
            <section className="mb-10 grid gap-4 md:grid-cols-2">
              {derivedMetrics.map((metric) => (
                <div key={metric.metric_id} className="rounded-2xl bg-[#f7df1e] p-6 text-black">
                  <p className="ff-micro text-sm font-bold opacity-70">
                    {derivedMetricLabel(metric.metric_id)}
                  </p>
                  <p className="ff-mono-num mt-2 text-5xl font-black">
                    {metric.metric_id.includes("percent")
                      ? `${metric.value.toFixed(2)}%`
                      : formatMoney(metric.value)}
                  </p>
                  <p className="mt-3 text-xs font-medium opacity-60">
                    Formula: {metric.formula}
                  </p>
                  <p className="mt-1 text-xs font-medium opacity-50">
                    Vintage: {metric.vintage_date}. Uses latest available observations by metric.
                  </p>
                </div>
              ))}
            </section>
          )}

          <section>
            <h2 className="text-xl font-bold mb-4">Component Prices</h2>
            {foodMetrics.length === 0 ? (
              <p className="text-sm text-slate-500">No food price data has been imported yet.</p>
            ) : (
              <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
                {foodMetrics.map((item) => (
                  <div key={item.metric.metric_id} className="ff-terminal rounded-xl p-5">
                    <p className="text-sm font-semibold text-slate-300">{item.metric.name}</p>
                    <p className="ff-mono-num mt-2 text-3xl font-black">
                      {formatMetricValue(item.observation.value, item.metric.unit)}
                    </p>
                    <div className="mt-3 space-y-1">
                      <p className="text-[11px] text-slate-500">
                        Date: {item.observation.date}
                      </p>
                      <p className="text-[11px] text-slate-500">
                        Series: {item.observation.source_series_id}
                      </p>
                      <p className="text-[11px] text-slate-500">
                        Scope: {item.observation.notes}
                      </p>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </section>

          <section className="ff-blueprint mt-10 rounded-xl p-6">
            <h2 className="text-lg font-bold mb-3">Basket Methodology</h2>
            <ul className="space-y-2 text-sm text-slate-400 leading-relaxed">
              <li>The food basket represents one unit of each item: 1 dozen eggs, 1 gallon milk (whole fortified), 1 lb white pan bread, 1 lb 100% ground beef, 1 lb boneless chicken breast, 1 lb bananas.</li>
              <li>All component prices come from the BLS Average Price Data program (APU series), U.S. city average.</li>
              <li>CPI year-over-year changes use the unadjusted CPI-U series for food at home (SAF11) and food away from home (SEFV).</li>
              <li>No regional variation, state-level grocery prices, or household consumption weights are applied.</li>
              <li>Food basket share of weekly wage uses national average hourly earnings (CES) at 40 hours per week.</li>
            </ul>
          </section>
        </>
      )}
    </main>
  );
}
