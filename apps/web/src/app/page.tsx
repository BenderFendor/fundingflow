import Link from "next/link";
import { HomeSearch } from "./home-search";
import { SiteHeader } from "@/components/site-header";
import { SiteFooter } from "@/components/site-footer";
import {
  CropMarks,
  DataBadge,
  Decimals,
  EmptyState,
  MetricTile,
  SectionHeading,
} from "@/components/ui";
import { formatMetricValue, formatMoney, getNationalPulse } from "@/lib/types";

const featuredStates = [
  { code: "PA", name: "Pennsylvania", focus: "Awards, rent, wages", tone: "bg-[#ee744f]" },
  { code: "CA", name: "California", focus: "Labor and housing", tone: "bg-[#d0f1ed]" },
  { code: "TX", name: "Texas", focus: "Energy and contracts", tone: "bg-[#f8df1d]" },
  { code: "NY", name: "New York", focus: "Income and public money", tone: "bg-[#aa9fda]" },
];

const pulseBars = [44, 70, 38, 86, 58, 74, 48, 62, 91, 53, 76, 66];

const sourceLanes = [
  ["USAspending", "Federal awards"],
  ["LDA", "Lobbying filings"],
  ["FEC", "Campaign finance"],
  ["SEC", "Corporate disclosures"],
  ["BLS + Census", "Economic pressure"],
];

const researchPaths = [
  {
    index: "01",
    title: "Follow an organization",
    body: "Open an entity record to inspect identifiers, awards, lobbying filings, campaign-finance records, and documented relationships.",
    query: "Lockheed Martin",
  },
  {
    index: "02",
    title: "Read a state economy",
    body: "Compare public obligations with wages, labor conditions, housing pressure, food prices, and energy costs by geography.",
    query: "Pennsylvania",
  },
  {
    index: "03",
    title: "Audit the evidence chain",
    body: "Review how source records become evidence, canonical entities, metrics, and neutral relationship edges.",
    href: "/data",
  },
];

export default async function HomePage({
  searchParams,
}: {
  searchParams: Promise<{ q?: string }>;
}) {
  const { q = "" } = await searchParams;
  let pulse = null;
  try {
    pulse = await getNationalPulse();
  } catch {
    // Search and documentation remain usable while the API is unavailable.
  }

  const value = (metricId: string): number | null => {
    const match = pulse?.latest_metrics.find((item) => item.observation.metric_id === metricId);
    return match?.observation.value ?? null;
  };

  const nationalCards = pulse
    ? [
        { label: "Unemployment", value: value("unemployment_rate"), unit: "percent" },
        { label: "Payroll jobs change", value: value("payroll_jobs_change"), unit: "jobs" },
        { label: "Average hourly earnings", value: value("avg_hourly_earnings"), unit: "usd_per_hour" },
        { label: "Median weekly earnings", value: value("median_weekly_earnings"), unit: "usd_per_week" },
        { label: "Food at home CPI YoY", value: value("food_at_home_cpi_yoy"), unit: "percent" },
        { label: "Ground beef", value: value("ground_beef_price"), unit: "usd_per_lb" },
        { label: "Chicken breast", value: value("chicken_breast_price"), unit: "usd_per_lb" },
        { label: "Milk", value: value("milk_price"), unit: "usd_per_gallon" },
        { label: "Bread", value: value("white_bread_price"), unit: "usd_per_lb" },
        { label: "Gasoline", value: value("regular_gas_price"), unit: "usd_per_gallon" },
      ]
    : [];

  const derivedCards = (pulse?.derived_metrics ?? []).map((metric) => ({
    id: metric.metric_id,
    label: metric.metric_id === "food_basket_cost" ? "Representative food basket" : "Basket share of weekly wage",
    value: metric.value,
    formula: metric.formula,
    vintage: metric.vintage_date,
    unit: metric.metric_id === "food_basket_cost" ? "usd" : "percent",
  }));

  const heroMetric = derivedCards.find((metric) => metric.id === "food_basket_cost") ?? derivedCards[0];
  const vintage = pulse?.latest_metrics[0]?.observation.vintage_date ?? "API offline";

  return (
    <div className="min-h-screen text-white">
      <SiteHeader />

      <main id="main-content" className="mx-auto w-full max-w-[1440px] px-4 pb-16 pt-4 sm:px-6 lg:px-8">
        <section className="grid border border-white/10 lg:grid-cols-12">
          <div className="ff-paper ff-enter relative overflow-hidden p-7 sm:p-10 lg:col-span-8 lg:min-h-[650px] lg:p-14">
            <CropMarks className="text-black/35" />
            <div className="absolute -bottom-16 right-0 select-none font-mono text-[14rem] font-black leading-none tracking-[-0.12em] text-black/[0.035] sm:text-[20rem] lg:text-[24rem]">
              FF
            </div>

            <div className="relative z-10 flex min-h-full flex-col justify-between">
              <div>
                <div className="flex flex-wrap items-center gap-3">
                  <p className="ff-kicker text-black/55">Public records graph</p>
                  <DataBadge tone="paper">Evidence first</DataBadge>
                  <DataBadge tone="paper">No causal claims</DataBadge>
                </div>
                <h1 className="ff-sans-wide mt-10 max-w-4xl text-5xl font-black leading-[0.84] tracking-[-0.075em] text-black sm:text-7xl lg:text-[6.6rem]">
                  Trace money, institutions, and public power.
                </h1>
                <p className="mt-8 max-w-2xl text-base font-medium leading-7 text-black/62 sm:text-lg">
                  Search federal awards, lobbying disclosures, campaign finance, corporate records, and economic conditions through one source-backed entity graph.
                </p>
              </div>

              <div className="mt-14 max-w-4xl">
                <HomeSearch initialQuery={q} />
              </div>
            </div>
          </div>

          <aside className="ff-terminal ff-scanline ff-enter ff-enter-delay-1 relative flex flex-col lg:col-span-4">
            <div className="relative flex-1 border-b border-white/10 p-7 sm:p-9">
              <CropMarks className="text-white/20" />
              <div className="flex items-center justify-between gap-4">
                <p className="ff-kicker text-[#dfff00]">National pulse</p>
                <DataBadge>{vintage}</DataBadge>
              </div>

              <div className="mt-12">
                <p className="ff-micro text-[9px] font-black text-slate-600">Latest derived indicator</p>
                <p className="ff-pixel mt-4 text-[4.8rem] font-black leading-none tracking-[-0.1em] text-white sm:text-[6rem]">
                  {heroMetric ? (
                    <Decimals>
                      {heroMetric.unit === "usd" ? formatMoney(heroMetric.value) : `${heroMetric.value.toFixed(2)}%`}
                    </Decimals>
                  ) : "--"}
                </p>
                <p className="mt-4 max-w-sm text-sm leading-6 text-slate-500">
                  {heroMetric?.label ?? "National metrics have not been imported."}
                </p>
              </div>

              <div className="mt-12 flex h-28 items-end gap-1 border-b border-l border-white/10 px-2 pt-4">
                {pulseBars.map((height, index) => (
                  <span
                    key={`pulse-${index}`}
                    className="flex-1 bg-[#dfff00]"
                    style={{ height: `${height}%`, opacity: index === pulseBars.length - 1 ? 1 : 0.18 + (index % 4) * 0.08 }}
                  />
                ))}
              </div>
              <div className="mt-3 flex justify-between font-mono text-[8px] uppercase tracking-widest text-slate-700">
                <span>Historical window</span>
                <span>Latest</span>
              </div>
            </div>

            <div className="p-7 sm:p-9">
              <p className="ff-micro text-[9px] font-black text-slate-600">Connected source lanes</p>
              <div className="mt-5 divide-y divide-white/10 border-y border-white/10">
                {sourceLanes.map(([source, scope], index) => (
                  <div key={source} className="grid grid-cols-[2rem_1fr_auto] items-center gap-3 py-3.5">
                    <span className="font-mono text-[9px] text-white/20">{String(index + 1).padStart(2, "0")}</span>
                    <span className="text-sm font-bold text-slate-300">{source}</span>
                    <span className="font-mono text-[8px] uppercase tracking-wider text-slate-600">{scope}</span>
                  </div>
                ))}
              </div>
            </div>
          </aside>
        </section>

        <section className="mt-4 grid border border-white/10 bg-black/30 sm:grid-cols-2 lg:grid-cols-4">
          {featuredStates.map((state, index) => (
            <Link
              key={state.code}
              href={`/states/${state.code}`}
              className="ff-focus-ring ff-interactive group relative min-h-52 overflow-hidden border-r border-b border-white/10 bg-black/25 p-6 hover:bg-white/[0.045]"
            >
              <CropMarks className="text-white/15" />
              <div className="flex items-center justify-between">
                <span className={`h-3 w-3 ${state.tone}`} />
                <span className="font-mono text-[9px] text-slate-700">0{index + 1}</span>
              </div>
              <p className="ff-pixel mt-10 text-5xl font-black tracking-[-0.1em] text-white group-hover:text-[#dfff00]">{state.code}</p>
              <p className="mt-5 text-sm font-bold text-slate-300">{state.name}</p>
              <p className="mt-1 font-mono text-[8px] uppercase tracking-widest text-slate-600">{state.focus}</p>
            </Link>
          ))}
        </section>

        {pulse ? (
          <section className="ff-terminal mt-8 overflow-hidden">
            <SectionHeading
              eyebrow="National evidence board"
              title="Household pressure and labor conditions"
              description="Latest imported national observations. Each value retains its source, date, and vintage in the underlying record."
              meta={<DataBadge tone="acid">{pulse.latest_metrics.length} current metrics</DataBadge>}
            />

            {derivedCards.length > 0 ? (
              <div className="grid border-b border-white/10 md:grid-cols-2">
                {derivedCards.map((card, index) => (
                  <article
                    key={card.id}
                    className={`relative overflow-hidden border-r border-b border-black/15 p-7 text-black sm:p-9 ${index % 2 === 0 ? "bg-[#d0f1ed]" : "bg-[#aa9fda]"}`}
                  >
                    <CropMarks className="text-black/35" />
                    <div className="relative z-10 flex items-start justify-between gap-4">
                      <DataBadge tone="paper">Derived metric</DataBadge>
                      <span className="font-mono text-[9px] uppercase tracking-wider text-black/40">{card.vintage}</span>
                    </div>
                    <p className="ff-sans-wide relative z-10 mt-8 max-w-lg text-xl font-black">{card.label}</p>
                    <p className="ff-pixel relative z-10 mt-5 text-5xl font-black leading-none tracking-[-0.09em] sm:text-6xl">
                      <Decimals>{card.unit === "usd" ? formatMoney(card.value) : `${card.value.toFixed(2)}%`}</Decimals>
                    </p>
                    <p className="relative z-10 mt-8 max-w-xl border-t border-black/15 pt-4 font-mono text-[9px] uppercase leading-5 tracking-wider text-black/55">
                      {card.formula}
                    </p>
                  </article>
                ))}
              </div>
            ) : null}

            <div className="grid sm:grid-cols-2 lg:grid-cols-5">
              {nationalCards.map((card, index) => (
                <MetricTile
                  key={card.label}
                  label={card.label}
                  value={card.value == null ? "--" : <Decimals>{formatMetricValue(card.value, card.unit)}</Decimals>}
                  meta={index < 5 ? "National observation" : "U.S. city average"}
                  accent={index === 0 ? "orange" : index === 4 ? "yellow" : "white"}
                />
              ))}
            </div>

            <div className="border-t border-white/10 px-6 py-5 sm:px-8">
              <p className="max-w-5xl font-mono text-[9px] uppercase leading-5 tracking-wider text-slate-600">
                Food item prices use BLS U.S. city averages and are not state grocery estimates. Federal obligations are public-record amounts, not recognized company revenue.
              </p>
            </div>
          </section>
        ) : (
          <section className="mt-8">
            <EmptyState
              title="National pulse is not available"
              description="The frontend is online, but the API or economic imports are unavailable. Search will resume when the data service is ready; methodology and data-lineage pages remain available."
              action={
                <Link href="/data" className="ff-focus-ring inline-flex bg-[#dfff00] px-4 py-2 font-mono text-[9px] font-black uppercase tracking-wider text-black hover:bg-[#efff68]">
                  Inspect data pipeline
                </Link>
              }
            />
          </section>
        )}

        <section className="mt-8 border border-white/10 bg-black/25">
          <SectionHeading
            eyebrow="Research paths"
            title="Start with a question, not a dashboard"
            description="FundingFlow organizes public records around traceable entities, geographies, and evidence chains."
            meta={<DataBadge>3 entry points</DataBadge>}
          />
          <div className="grid lg:grid-cols-3">
            {researchPaths.map((path) => {
              const href = path.href ?? `/?q=${encodeURIComponent(path.query ?? "")}`;
              return (
                <Link
                  key={path.index}
                  href={href}
                  className="ff-focus-ring group relative min-h-72 overflow-hidden border-r border-b border-white/10 p-7 transition-colors hover:bg-white/[0.04] sm:p-8"
                >
                  <CropMarks className="text-white/15" />
                  <p className="ff-pixel text-5xl font-black text-white/[0.08]">{path.index}</p>
                  <h3 className="ff-sans-wide mt-10 text-2xl font-black text-white group-hover:text-[#dfff00]">{path.title}</h3>
                  <p className="mt-4 max-w-md text-sm leading-6 text-slate-500">{path.body}</p>
                  <span className="mt-8 inline-flex font-mono text-[9px] font-black uppercase tracking-widest text-slate-600 group-hover:text-white">
                    Open path <span className="ml-2">→</span>
                  </span>
                </Link>
              );
            })}
          </div>
        </section>
      </main>

      <SiteFooter />
    </div>
  );
}
