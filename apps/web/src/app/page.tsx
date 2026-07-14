import Link from "next/link";
import { getNationalPulse, formatMoney, formatMetricValue } from "@/lib/types";
import { HomeSearch } from "./home-search";
import { CropMarks, Decimals } from "@/components/ui";

// Public Ledger Terminal: dense financial UI with oversized evidence numbers and simple comparison graphics.
const featuredStates = [
  { code: "PA", name: "Pennsylvania", focus: "Federal awards, rent, wages", accent: "bg-[#e96b4c] text-black" },
  { code: "CA", name: "California", focus: "Labor markets and housing pressure", accent: "bg-[#d2f1ec] text-black" },
  { code: "TX", name: "Texas", focus: "Energy, contracts, and payrolls", accent: "bg-[#f8df1d] text-black" },
  { code: "NY", name: "New York", focus: "Income, rents, and public money", accent: "bg-[#a69ad1] text-black" },
];

const pulseBars = [44, 70, 38, 86, 58, 74, 48, 62, 91, 53, 76, 66];

export default async function HomePage() {
  let pulse = null;
  try {
    pulse = await getNationalPulse();
  } catch {
    // The page still works as a search shell when the local API is offline.
  }

  const value = (metricId: string): number | null => {
    if (!pulse) return null;
    const match = pulse.latest_metrics.find(
      (item) => item.observation.metric_id === metricId,
    );
    return match?.observation.value ?? null;
  };

  const nationalCards = pulse
    ? [
        { label: "Unemployment", value: value("unemployment_rate"), unit: "percent", href: null },
        { label: "Payroll jobs change", value: value("payroll_jobs_change"), unit: "jobs", href: null },
        { label: "Avg hourly earnings", value: value("avg_hourly_earnings"), unit: "usd_per_hour", href: null },
        { label: "Median weekly earnings", value: value("median_weekly_earnings"), unit: "usd_per_week", href: null },
        { label: "Food at home CPI YoY", value: value("food_at_home_cpi_yoy"), unit: "percent", href: "/cost-basket" },
        { label: "Ground beef", value: value("ground_beef_price"), unit: "usd_per_lb", href: "/cost-basket" },
        { label: "Chicken breast", value: value("chicken_breast_price"), unit: "usd_per_lb", href: "/cost-basket" },
        { label: "Milk", value: value("milk_price"), unit: "usd_per_gallon", href: "/cost-basket" },
        { label: "Bread", value: value("white_bread_price"), unit: "usd_per_lb", href: "/cost-basket" },
        { label: "Gasoline", value: value("regular_gas_price"), unit: "usd_per_gallon", href: null },
      ]
    : [];

  const derivedCards = pulse?.derived_metrics
    ? pulse.derived_metrics.map((d) => ({
        label: d.metric_id === "food_basket_cost" ? "Food basket" : "Basket share",
        value: d.value,
        unit: d.metric_id === "food_basket_cost" ? "usd" : "percent",
        formula: d.formula,
        vintage: d.vintage_date,
      }))
    : [];
  const heroMetric = derivedCards[0];
  const vintage = pulse?.latest_metrics[0]?.observation.vintage_date ?? "offline";

  return (
    <div className="min-h-screen text-white font-sans">
      <header className="mx-auto flex w-full max-w-7xl flex-col gap-4 px-4 py-8 sm:flex-row sm:items-center sm:justify-between">
        <Link href="/" className="ff-sans-wide text-3xl tracking-tighter text-white hover:opacity-70 ff-fluid">
          FundingFlow
        </Link>
        <nav className="flex flex-wrap items-center gap-2 text-xs font-semibold">
          {featuredStates.map((state) => (
            <Link
              key={state.code}
              href={`/states/${state.code}`}
              className="ff-focus-ring font-mono border border-white/10 bg-white/5 hover:bg-white/10 px-4 py-2 rounded-full text-slate-300 ff-fluid"
            >
              {state.code}
            </Link>
          ))}
          <Link href="/cost-basket" className="ff-focus-ring font-mono border border-white/10 bg-white/5 hover:bg-white/10 px-4 py-2 rounded-full text-slate-300 ff-fluid">
            FOOD PRICES
          </Link>
          <Link href="/data" className="ff-focus-ring font-mono border border-white/10 bg-white/5 hover:bg-white/10 px-4 py-2 rounded-full text-slate-300 ff-fluid">
            DATA LINEAGE
          </Link>
          <Link href="/methodology" className="ff-focus-ring font-mono bg-[#f8df1d] text-black px-5 py-2 rounded-full transition-colors hover:bg-amber-400 ff-fluid">
            METHODOLOGY
          </Link>
        </nav>
      </header>

      <main className="mx-auto w-full max-w-7xl px-4 pb-16">
        <div className="grid grid-cols-1 lg:grid-cols-12 border border-white/10 bg-black">
          <section className="relative overflow-hidden p-8 sm:p-12 lg:col-span-8 lg:min-h-[560px] border-b lg:border-b-0 lg:border-r border-white/10">
            <CropMarks className="text-white/10" />
            <div className="ff-serif absolute -right-4 bottom-2 hidden text-[28rem] leading-none text-white/[0.02] lg:block select-none pointer-events-none">
              01
            </div>
            <div className="relative z-10 flex min-h-[500px] flex-col justify-between">
              <div className="max-w-3xl">
                <span className="font-mono text-[10px] font-black text-[#DFFF00] uppercase tracking-widest">
                  Public records graph
                </span>
                <h1 className="ff-sans-wide mt-6 text-5xl font-black leading-[0.9] tracking-tighter sm:text-7xl lg:text-8xl text-white">
                  Trace the flow of federal power.
                </h1>
                <p className="mt-8 max-w-2xl text-base font-medium leading-relaxed text-slate-400 sm:text-lg">
                  Search entities, states, awards, lobbying filings, wage pressure, food costs, and energy signals from one source-backed workspace.
                </p>
              </div>
              <div className="mt-12">
                <HomeSearch />
              </div>
            </div>
          </section>

          <aside className="grid grid-cols-1 lg:col-span-4 content-start">
            <div className="relative overflow-hidden p-8 text-white border-b border-white/10">
              <CropMarks className="text-white/10" />
              <p className="font-mono text-[10px] font-black text-[#DFFF00] uppercase tracking-widest">
                National pulse
              </p>
              <div className="mt-8">
                <p className="ff-pixel text-[6rem] sm:text-[7rem] tracking-tight leading-none text-white">
                  {heroMetric ? (
                    <Decimals>
                      {heroMetric.unit === "usd" ? formatMoney(heroMetric.value) : `${heroMetric.value.toFixed(2)}%`}
                    </Decimals>
                  ) : "--"}
                </p>
                <p className="mt-3 font-mono text-[10px] font-medium text-slate-500 uppercase tracking-widest">
                  {heroMetric?.label ?? "API offline"} / vintage {vintage}
                </p>
              </div>
              <div className="mt-8 flex h-20 items-end gap-[1px]">
                {pulseBars.map((height, index) => (
                  <div
                    key={`pulse-bar-${index}`}
                    className="flex-1 bg-[#DFFF00] transition-opacity hover:opacity-100"
                    style={{ height: `${height}%`, opacity: index % 3 === 0 ? 0.8 : 0.2 }}
                  />
                ))}
              </div>
            </div>

            <div className="relative overflow-hidden p-8">
              <CropMarks className="text-white/10" />
              <p className="font-mono text-[10px] font-black text-[#DFFF00] uppercase tracking-widest">
                Default state deck
              </p>
              <div className="mt-8 grid grid-cols-2 gap-[1px] bg-white/10 border border-white/10 p-[1px]">
                {featuredStates.map((state) => (
                  <Link
                    key={state.code}
                    href={`/states/${state.code}`}
                    className="ff-focus-ring group relative overflow-hidden bg-black p-5 transition-colors hover:bg-white/5"
                  >
                    <p className="ff-pixel text-[4rem] tracking-tighter leading-none text-white">{state.code}</p>
                    <p className="mt-4 font-mono text-[10px] font-black uppercase tracking-widest text-[#DFFF00]">{state.name}</p>
                    <p className="mt-1 font-mono text-[9px] font-medium opacity-50 uppercase tracking-wider text-slate-400">{state.focus}</p>
                  </Link>
                ))}
              </div>
            </div>
          </aside>
        </div>
        {pulse ? (
          <section className="mt-8 border border-white/10 bg-black">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-end sm:justify-between p-8 border-b border-white/10">
              <div>
                <p className="ff-micro text-[10px] font-black text-[#DFFF00] uppercase tracking-widest">
                  Evidence board
                </p>
                <h2 className="ff-sans-wide text-4xl font-black tracking-tighter text-white mt-2">National conditions</h2>
              </div>
              <span className="font-mono text-[10px] uppercase tracking-wider text-slate-500">
                {pulse.geo.name} / {pulse.latest_metrics.length} latest metrics
              </span>
            </div>

            {derivedCards.length > 0 && (
              <div className="grid grid-cols-1 md:grid-cols-2 border-b border-white/10 bg-white/10 gap-[1px]">
                {derivedCards.map((card, index) => (
                  <div
                    key={card.label}
                    className="relative overflow-hidden p-8 text-white bg-black transition-colors hover:bg-white/5"
                  >
                    <CropMarks className="opacity-20" />
                    <span className="ff-micro text-[10px] font-black opacity-80 uppercase tracking-widest text-[#DFFF00]">{card.label}</span>
                    <p className="ff-pixel mt-6 text-[5rem] tracking-tighter leading-none">
                      <Decimals>
                        {card.unit === "usd" ? formatMoney(card.value) : `${card.value.toFixed(2)}%`}
                      </Decimals>
                    </p>
                    <p className="mt-8 max-w-xl font-mono text-[10px] font-medium opacity-60 leading-relaxed uppercase tracking-wider">{card.formula}</p>
                  </div>
                ))}
              </div>
            )}

            <div className="grid grid-cols-2 lg:grid-cols-5 gap-[1px] bg-white/10">
              {nationalCards.map((card, index) => {
                const content = (
                  <div className="relative overflow-hidden min-h-48 p-6 transition-colors hover:bg-white/5 bg-black flex flex-col justify-between">
                    <CropMarks className="text-white/10" />
                    <div>
                      <p className="ff-micro text-[9px] font-black text-slate-400 leading-tight uppercase tracking-widest">
                        {card.label}
                      </p>
                      <p className="ff-pixel mt-4 text-[2.5rem] tracking-tighter leading-none text-white">
                        {card.value != null ? (
                          <Decimals>
                            {formatMetricValue(card.value, card.unit)}
                          </Decimals>
                        ) : "--"}
                      </p>
                    </div>
                    <div className="mt-8 flex h-8 items-end gap-[1px]">
                      {[36, 58, 45, 76, 62, 88].map((bar, barIndex) => (
                        <span
                          key={`${card.label}-${barIndex}`}
                          className="flex-1 bg-white/20"
                          style={{ height: `${Math.max(16, bar - index * 2)}%` }}
                        />
                      ))}
                    </div>
                  </div>
                );
                return card.href ? (
                  <Link key={card.label} href={card.href} className="block">
                    {content}
                  </Link>
                ) : (
                  <div key={card.label}>{content}</div>
                );
              })}
            </div>
            <div className="p-6 border-t border-white/10">
              <p className="font-mono text-[9px] uppercase tracking-widest leading-relaxed text-slate-600 max-w-4xl">
                BLS CPI-U U.S. city average and average price data. Food items are national, not state-level. Petroleum prices are U.S. regular conventional retail average.
              </p>
            </div>
          </section>
        ) : (
          <section className="relative overflow-hidden border border-dashed border-white/20 p-10 text-center text-[10px] font-mono uppercase tracking-widest text-slate-500 lg:col-span-12 mt-8">
            <CropMarks className="text-slate-800" />
            National pulse data is unavailable. Start the API server and run economic imports to populate live metrics.
          </section>
        )}
      </main>
    </div>
  );
}
