import Link from "next/link";
import {
  derivedMetricLabel,
  formatMetricValue,
  formatMoney,
  getMetricHistory,
  getNationalPulse,
  getStateProfile,
} from "@/lib/types";
import type { MetricObservationWithDetails } from "@/lib/types";
import { CropMarks, Decimals } from "@/components/ui";
// Public Ledger Terminal: state profiles use block charts and ranked ledgers for fast source-backed comparison.
type StatePageProps = {
  params: Promise<{ stateCode: string }>;
};

const sections = [
  { key: "labor", title: "Labor" },
  { key: "wages", title: "Wages" },
  { key: "housing", title: "Housing" },
  { key: "food", title: "Food" },
  { key: "energy", title: "Energy" },
  { key: "economy", title: "Economic Structure" },
  { key: "contracts", title: "Public Money" },
  { key: "income", title: "Income And Poverty" },
];

const chartableMetricIds = [
  "unemployment_rate",
  "avg_hourly_earnings",
  "fhfa_hpi_yoy",
  "regular_gas_price",
  "fmr_2br",
  "median_gross_rent",
];

function BlockHistory({
  series,
  unit,
  accent = "bg-[#d9f7f2]",
}: {
  series: MetricObservationWithDetails[];
  unit: string;
  accent?: string;
}) {
  const values = series.map((s) => s.observation.value);
  const min = Math.min(...values);
  const max = Math.max(...values);
  const range = max - min || 1;
  const latest = series[series.length - 1];

  return (
    <>
      <div className="mt-5 flex h-28 items-end gap-1">
        {series.slice(-36).map((s, i) => (
          <div
            key={`${s.observation.date}-${i}`}
            className={`flex-1 rounded-t-sm ${accent} transition hover:opacity-100`}
            style={{
              height: `${Math.max(((s.observation.value - min) / range) * 100, 8)}%`,
              opacity: i === series.slice(-36).length - 1 ? 1 : 0.45,
            }}
            title={`${s.observation.date}: ${formatMetricValue(s.observation.value, unit)}`}
          />
        ))}
      </div>
      <div className="mt-3 flex items-center justify-between font-mono text-[9px] font-bold uppercase tracking-[0.14em] text-slate-500">
        <span>{series[0].observation.date}</span>
        <span>{latest.observation.date}</span>
      </div>
    </>
  );
}

export default async function StateProfilePage({ params }: StatePageProps) {
  const { stateCode } = await params;
  const [profile, nationalPulse] = await Promise.all([
    getStateProfile(stateCode.toUpperCase()),
    getNationalPulse(),
  ]);

  const availableChartMetrics = chartableMetricIds.filter((id) =>
    profile.latest_metrics.some((item) => item.observation.metric_id === id),
  );
  const stateCodeUpper = stateCode.toUpperCase();
  const chartHistoryPromises = availableChartMetrics.map((metricId) => {
    const latest = profile.latest_metrics.find(
      (item) => item.observation.metric_id === metricId,
    );

    return getMetricHistory(stateCodeUpper, metricId)
      .then((history) => ({
        metricId,
        label: latest?.metric.name ?? metricId,
        unit: latest?.metric.unit ?? "usd",
        history,
      }))
      .catch(() => ({
        metricId,
        label: metricId,
        unit: "usd",
        history: [],
      }));
  });
  const chartHistories = await Promise.all(chartHistoryPromises);
  const publicMoneyMetric = profile.latest_metrics.find(
    (item) => item.metric.metric_id === "federal_contract_obligations",
  );
  const displayedObligations =
    profile.public_money.total_obligations || publicMoneyMetric?.observation.value || 0;

  const metricsByCategory = new Map(
    sections.map((section) => [
      section.key,
      profile.latest_metrics.filter((item) => item.metric.category === section.key),
    ]),
  );
  const nationalFoodMetrics = nationalPulse.latest_metrics.filter(
    (item) => item.metric.category === "food",
  );
  const nationalFoodDerivedMetrics = (nationalPulse.derived_metrics ?? []).filter((metric) =>
    metric.metric_id.startsWith("food_"),
  );
  const heroMetrics = [
    profile.latest_metrics.find((item) => item.observation.metric_id === "unemployment_rate"),
    profile.latest_metrics.find((item) => item.observation.metric_id === "avg_hourly_earnings"),
    profile.latest_metrics.find((item) => item.observation.metric_id === "fmr_2br"),
  ].filter(Boolean);

  return (
    <main className="min-h-screen px-4 py-6 text-slate-100">
      <div className="mx-auto max-w-7xl">
        <header className="grid gap-4 lg:grid-cols-12">
          <section className="ff-paper relative overflow-hidden rounded-[2rem] p-8 text-black sm:p-12 lg:col-span-8 ff-fluid hover:scale-[1.01]">
            <CropMarks className="text-black/15" />
            <div className="ff-serif absolute -right-4 top-4 hidden text-[18rem] leading-none text-black/[0.03] lg:block select-none pointer-events-none">
              {profile.geo.geo_id}
            </div>
            <Link href="/" className="ff-focus-ring font-mono text-[10px] font-bold uppercase tracking-wider text-black/50 hover:text-black ff-fluid bg-black/5 px-3 py-1.5 rounded-full inline-block">
              &larr; Back to search
            </Link>
            <div className="mt-12">
              <span className="ff-micro text-[10px] font-black text-black/50 bg-black/5 px-3 py-1.5 rounded-full inline-block">
                State economic conditions
              </span>
              <h1 className="ff-sans-wide mt-6 break-words text-6xl font-black leading-[0.88] tracking-tighter sm:text-8xl sm:leading-[0.86] text-black">
                {profile.geo.name}
              </h1>
              <p className="mt-8 max-w-3xl text-base font-medium leading-relaxed text-black/60 sm:text-lg">
                Public money, labor markets, rent pressure, food and energy costs, and state economic structure in one source-backed profile.
              </p>
            </div>
            <div className="mt-12 grid gap-4 sm:grid-cols-3">
              {heroMetrics.map((item) => (
                <div key={item!.metric.metric_id} className="relative overflow-hidden rounded-[1.5rem] bg-black/[0.03] border border-black/5 p-6 ff-fluid hover:scale-105 hover:bg-black/[0.06]">
                  <CropMarks className="opacity-10" />
                  <p className="ff-micro text-[9px] font-black text-black/50 leading-none bg-black/5 px-2 py-1 rounded-full inline-block">
                    {item!.metric.name}
                  </p>
                  <p className="ff-pixel mt-6 text-[3rem] font-bold leading-none text-black">
                    <Decimals>{formatMetricValue(item!.observation.value, item!.metric.unit)}</Decimals>
                  </p>
                  <p className="mt-4 font-mono text-[10px] font-bold text-black/45 leading-none">
                    {item!.source.agency} / {item!.observation.date}
                  </p>
                </div>
              ))}
            </div>
          </section>

          <aside className="grid gap-4 lg:col-span-4">
            <div className="relative overflow-hidden rounded-[2rem] bg-[#f8df1d] p-8 text-black ff-fluid hover:scale-[1.02]">
              <CropMarks className="opacity-20" />
              <span className="ff-micro text-[10px] font-black text-black/60 bg-black/5 px-3 py-1.5 rounded-full inline-block">
                Federal obligations
              </span>
              <p className="ff-pixel mt-8 text-[5rem] sm:text-[6rem] tracking-tighter font-bold leading-none text-black">
                <Decimals>{formatMoney(displayedObligations)}</Decimals>
              </p>
              <p className="mt-6 text-[11px] font-semibold text-black/60 leading-tight">
                {publicMoneyMetric
                  ? `${publicMoneyMetric.source.agency} / ${publicMoneyMetric.source.dataset}`
                  : "Public money summary"}
              </p>
            </div>

            <div className="ff-blueprint relative overflow-hidden rounded-[1.75rem] p-6">
              <CropMarks className="text-white/20" />
              <p className="ff-micro text-xs font-black text-[#f5f2e8]/70">
                Coverage
              </p>
              <div className="mt-5 grid grid-cols-3 gap-2.5 text-center">
                <div className="relative overflow-hidden rounded-2xl border border-white/12 bg-white/[0.04] p-4">
                  <CropMarks className="opacity-15" />
                  <p className="ff-pixel text-3xl font-black leading-none text-[#f8df1d]">
                    {profile.latest_metrics.length}
                  </p>
                  <p className="ff-micro mt-2 text-[9px] font-black text-[#f5f2e8]/60 leading-none">Metrics</p>
                </div>
                <div className="relative overflow-hidden rounded-2xl border border-white/12 bg-white/[0.04] p-4">
                  <CropMarks className="opacity-15" />
                  <p className="ff-pixel text-3xl font-black leading-none text-[#f8df1d]">
                    {profile.derived_metrics.length}
                  </p>
                  <p className="ff-micro mt-2 text-[9px] font-black text-[#f5f2e8]/60 leading-none">Derived</p>
                </div>
                <div className="relative overflow-hidden rounded-2xl border border-white/12 bg-white/[0.04] p-4">
                  <CropMarks className="opacity-15" />
                  <p className="ff-pixel text-3xl font-black leading-none text-[#f8df1d]">
                    {profile.public_money.top_recipients.length}
                  </p>
                  <p className="ff-micro mt-2 text-[9px] font-black text-[#f5f2e8]/60 leading-none">Recipients</p>
                </div>
              </div>
            </div>
          </aside>
        </header>

        <section className="mt-4 grid gap-4 md:grid-cols-3">
          {profile.derived_metrics.length === 0 ? (
            <div className="relative overflow-hidden rounded-[2rem] border border-white/5 bg-white/[0.02] p-8 text-center text-sm font-mono text-slate-500 md:col-span-3">
              <CropMarks className="text-slate-800" />
              Derived pressure metrics are not available yet for this geography.
            </div>
          ) : (
            profile.derived_metrics.map((metric) => {
              const isIntensity = metric.metric_id.includes("intensity");
              const colorBg = isIntensity
                ? "bg-[#9cb199]"
                : metric.metric_id.includes("min")
                  ? "bg-[#e96b4c]"
                  : "bg-[#a69ad1]";

              return (
                <article
                  key={metric.metric_id}
                  className={`relative overflow-hidden rounded-[2rem] p-8 text-black ff-fluid hover:scale-[1.02] ${colorBg}`}
                >
                  <CropMarks className="opacity-20" />
                  <span className="ff-micro text-[10px] font-black opacity-70 bg-black/10 px-3 py-1.5 rounded-full inline-block">
                    {derivedMetricLabel(metric.metric_id)}
                  </span>
                  <p className="ff-pixel mt-8 text-[4.5rem] tracking-tighter leading-none font-bold">
                    <Decimals>
                      {isIntensity
                        ? `${(metric.value * 100).toFixed(3)}%`
                        : metric.value.toFixed(1)}
                    </Decimals>
                  </p>
                  <p className="mt-4 text-[10px] font-semibold leading-relaxed opacity-65 border-t border-black/10 pt-3">
                    Formula: {metric.formula}. Vintage: {metric.vintage_date}.
                  </p>
                </article>
              );
            })
          )}
        </section>
        <section className="ff-terminal relative overflow-hidden mt-8 rounded-[2rem] p-8 sm:p-10 ff-fluid hover:bg-white/[0.01]">
          <CropMarks className="text-white/10" />
          <div className="flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
            <div>
              <span className="ff-micro text-[10px] font-black text-slate-500 bg-white/5 px-3 py-1.5 rounded-full inline-block">National context</span>
              <h2 className="ff-sans-wide mt-4 text-4xl font-black tracking-tighter text-white">Food price basket</h2>
              <p className="mt-2 text-sm font-medium text-slate-400">
                National BLS CPI and U.S. city average prices. Use as pressure context, not state grocery data.
              </p>
            </div>
            <span className="font-mono text-[10px] font-black uppercase tracking-wider text-slate-500 bg-white/5 px-3 py-1.5 rounded-full">
              Scope: {nationalPulse.geo.name}
            </span>
          </div>
          {nationalFoodMetrics.length === 0 ? (
            <p className="mt-8 text-sm text-slate-500 font-mono">
              No national food price metrics have been imported yet. Run the BLS CPI importer to populate this section.
            </p>
          ) : (
            <>
              {nationalFoodDerivedMetrics.length > 0 && (
                <div className="mt-8 grid gap-4 md:grid-cols-2">
                  {nationalFoodDerivedMetrics.map((metric) => (
                    <div key={metric.metric_id} className="relative overflow-hidden rounded-[1.75rem] p-6 text-black bg-[#d2f1ec] ff-fluid hover:scale-105">
                      <CropMarks className="opacity-20" />
                      <span className="ff-micro text-[10px] font-black opacity-70 bg-black/10 px-3 py-1.5 rounded-full inline-block">
                        {derivedMetricLabel(metric.metric_id)}
                      </span>
                      <p className="ff-pixel mt-6 text-[4.5rem] tracking-tight font-bold leading-none">
                        <Decimals>
                          {metric.metric_id.includes("percent")
                            ? `${metric.value.toFixed(2)}%`
                            : formatMoney(metric.value)}
                        </Decimals>
                      </p>
                      <p className="mt-4 font-mono text-[10px] font-semibold opacity-65 leading-tight">
                        Formula: {metric.formula}. Vintage: {metric.vintage_date}.
                      </p>
                    </div>
                  ))}
                </div>
              )}
              <div className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                {nationalFoodMetrics.map((item) => (
                  <div key={item.metric.metric_id} className="relative overflow-hidden rounded-[1.5rem] bg-black/40 border border-white/5 p-6 ff-fluid hover:bg-black/60 hover:border-white/10">
                    <CropMarks className="text-white/10" />
                    <p className="ff-micro text-[10px] font-black text-slate-400 leading-tight">{item.metric.name}</p>
                    <p className="ff-pixel mt-4 text-[3.5rem] font-bold leading-none text-white tracking-tighter">
                      <Decimals>{formatMetricValue(item.observation.value, item.metric.unit)}</Decimals>
                    </p>
                    <p className="mt-4 font-mono text-[10px] font-semibold text-slate-500 leading-none">
                      {item.observation.date} / {item.observation.source_series_id}
                    </p>
                  </div>
                ))}
              </div>
            </>
          )}
        </section>

        <section className="mt-4 grid gap-4 lg:grid-cols-2">
          {sections.map((section) => {
            const metrics = metricsByCategory.get(section.key) ?? [];
            return (
              <article key={section.key} className="relative overflow-hidden rounded-[2rem] border border-white/5 bg-white/[0.02] p-8 ff-fluid hover:bg-white/[0.04] hover:border-white/10">
                <CropMarks className="text-white/10" />
                <div className="flex items-center justify-between gap-3">
                  <h2 className="ff-sans-wide text-3xl font-black tracking-tighter text-white leading-none">{section.title}</h2>
                  <span className="ff-micro rounded-full bg-white/5 px-3 py-1.5 text-[10px] font-black text-slate-500">
                    {metrics.length} metrics
                  </span>
                </div>
                {metrics.length === 0 ? (
                  <div className="mt-8 rounded-[1.5rem] border border-dashed border-white/10 bg-black/20 p-6">
                    {section.key === "food" ? (
                      <>
                        <p className="text-sm font-semibold text-slate-300">
                          State-level food metrics are not imported yet.
                        </p>
                        <p className="mt-3 text-xs text-slate-500 leading-relaxed">
                          Use the national food price basket above for BLS U.S. city average prices. Do not read it as state grocery data.
                        </p>
                        <Link
                          href="/cost-basket"
                          className="ff-focus-ring mt-6 inline-block rounded-full bg-[#f8df1d] px-5 py-2 text-xs font-black uppercase tracking-wider text-black hover:bg-yellow-400 ff-fluid hover:scale-105"
                        >
                          Open basket
                        </Link>
                      </>
                    ) : (
                      <p className="text-sm font-semibold text-slate-500">
                        No sourced metric has been imported for this section yet.
                      </p>
                    )}
                  </div>
                ) : (
                  <div className="mt-8 divide-y divide-white/5">
                    {metrics.map((item) => (
                      <div key={item.metric.metric_id} className="py-4">
                        <div className="flex items-start justify-between gap-4">
                          <div>
                            <p className="text-base font-bold text-slate-200 tracking-tight">{item.metric.name}</p>
                            <p className="mt-1.5 font-mono text-[10px] font-medium text-slate-500 leading-none">
                              {item.source.agency} / {item.source.dataset}
                            </p>
                          </div>
                          <p className="ff-pixel shrink-0 text-3xl font-bold leading-none text-white tracking-tighter">
                            <Decimals>{formatMetricValue(item.observation.value, item.metric.unit)}</Decimals>
                          </p>
                        </div>
                        <p className="mt-3 font-mono text-[10px] text-slate-600 leading-none">
                          Date: {item.observation.date}. Vintage: {item.observation.vintage_date}.
                          {item.observation.notes ? ` ${item.observation.notes}` : ""}
                        </p>
                      </div>
                    ))}
                  </div>
                )}
              </article>
            );
          })}
        </section>

        <section className="relative overflow-hidden mt-4 rounded-[2rem] bg-[#111317] border border-white/5 p-8 sm:p-10 ff-fluid hover:bg-white/[0.03]">
          <CropMarks className="text-white/10" />
          <div className="flex flex-col gap-3 md:flex-row md:items-end md:justify-between">
            <div>
              <span className="ff-micro text-[10px] font-black text-slate-500 bg-white/5 px-3 py-1.5 rounded-full inline-block">Award ledger</span>
              <h2 className="ff-sans-wide mt-4 text-4xl font-black tracking-tighter text-white">Top public-money recipients</h2>
            </div>
            <p className="font-mono text-sm font-black text-[#f8df1d] bg-[#f8df1d]/10 px-4 py-2 rounded-full">
              {formatMoney(profile.public_money.total_obligations)} total linked obligations
            </p>
          </div>
          {profile.public_money.top_recipients.length === 0 ? (
            <div className="mt-8 font-mono text-sm text-slate-500">
              No federal award geography has been linked to this state yet.
              {publicMoneyMetric && (
                <p className="mt-2 text-xs text-slate-600">
                  Total federal obligations: {formatMoney(publicMoneyMetric.observation.value)}
                  ({publicMoneyMetric.observation.date})
                </p>
              )}
            </div>
          ) : (
            <div className="mt-10 space-y-4">
              {profile.public_money.top_recipients.map((recipient, index) => {
                const max = Math.max(
                  ...profile.public_money.top_recipients.map((r) => r.total_obligations),
                );
                const width = max > 0 ? Math.max((recipient.total_obligations / max) * 100, 4) : 4;
                return (
                  <div key={`${recipient.entity_id}-${recipient.display_name}`} className="relative overflow-hidden rounded-[1.5rem] bg-black/40 border border-white/5 p-6 ff-fluid hover:bg-black/60 hover:border-white/10">
                    <CropMarks className="text-white/10" />
                    <div className="flex items-start justify-between gap-4">
                      <div>
                        <p className="ff-micro text-[10px] font-black text-slate-500 bg-white/5 px-2 py-1 rounded-full inline-block mb-2">
                          Rank {index + 1}
                        </p>
                        <p className="text-lg font-bold text-white tracking-tight">{recipient.display_name}</p>
                        <p className="mt-2 font-mono text-[10px] font-semibold text-slate-500">
                          {recipient.award_count.toLocaleString()} award records
                        </p>
                      </div>
                      <p className="ff-pixel shrink-0 text-4xl font-bold text-white tracking-tighter">
                        <Decimals>{formatMoney(recipient.total_obligations)}</Decimals>
                      </p>
                    </div>
                    <div className="mt-6 h-2 overflow-hidden rounded-full bg-white/5">
                      <div className="h-full rounded-full bg-[#f8df1d]" style={{ width: `${width}%` }} />
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </section>

        <section className="mt-4 grid gap-4 lg:grid-cols-2">
          <div className="relative overflow-hidden rounded-[1.75rem] border-2 border-black bg-white/5 shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] p-5 sm:p-6">
            <CropMarks className="opacity-25" />
            <h2 className="ff-sans-wide text-2xl font-black text-white leading-none mt-1">Income distribution</h2>
            <p className="mt-2 text-sm font-semibold text-slate-500">
              Census ACS 2024 5-year state household income quintile means.
            </p>
            {(() => {
              const quintileLabels: Record<string, string> = {
                quintile_income_bottom: "Lowest 20%",
                quintile_income_second: "Second 20%",
                quintile_income_third: "Third 20%",
                quintile_income_fourth: "Fourth 20%",
                quintile_income_top: "Highest 20%",
              };
              const quintileMetrics = profile.latest_metrics.filter(
                (item) => item.observation.metric_id.startsWith("quintile_income_"),
              );
              const items = quintileMetrics
                .map((item) => ({
                  key: item.observation.metric_id,
                  label: quintileLabels[item.observation.metric_id] ?? item.metric.name,
                  value: item.observation.value,
                  date: item.observation.date,
                }))
                .sort((a, b) => {
                  const order = ["quintile_income_bottom", "quintile_income_second", "quintile_income_third", "quintile_income_fourth", "quintile_income_top"];
                  return order.indexOf(a.key) - order.indexOf(b.key);
                });
              if (items.length === 0) return <p className="mt-4 text-sm font-mono text-slate-500">No quintile income data imported yet.</p>;
              const max = Math.max(...items.map((q) => q.value));
              return (
                <div className="mt-5 space-y-4">
                  {items.map((q) => (
                    <div key={q.key}>
                      <div className="flex justify-between gap-4 text-sm font-semibold">
                        <span className="text-slate-300">{q.label}</span>
                        <span className="ff-pixel text-xl font-bold text-white">
                          <Decimals>{formatMoney(q.value)}</Decimals>
                        </span>
                      </div>
                      <div className="mt-2 h-4 overflow-hidden border border-black bg-white/10">
                        <div
                          className="h-full bg-[#d2f1ec]"
                          style={{ width: `${Math.max((q.value / max) * 100, 2)}%` }}
                        />
                      </div>
                    </div>
                  ))}
                  <p className="text-[10px] font-mono text-slate-600 leading-normal border-t border-white/5 pt-3 mt-4">
                    Top-quintile mean is derived from aggregate mean and lower-quintile sums. These are household means, not individual top-percentile thresholds.
                  </p>
                </div>
              );
            })()}
          </div>

          <div className="ff-terminal relative overflow-hidden rounded-[1.75rem] p-5 sm:p-6">
            <CropMarks className="text-slate-800 opacity-60" />
            <h2 className="ff-sans-wide text-2xl font-black text-white leading-none mt-1">Wage gaps</h2>
            <p className="mt-2 text-sm font-semibold text-slate-500">
              State ACS gender ratio and national BLS weekly earnings by race and gender when imported.
            </p>
            {(() => {
              const male = profile.latest_metrics.find(
                (item) => item.observation.metric_id === "median_earnings_male",
              );
              const female = profile.latest_metrics.find(
                (item) => item.observation.metric_id === "median_earnings_female",
              );
              const wageMetrics = nationalPulse?.latest_metrics?.filter(
                (item) => item.observation.metric_id.startsWith("median_weekly_earnings_"),
              ) ?? [];
              const metricById = new Map(
                wageMetrics.map((item) => [item.observation.metric_id, item]),
              );
              const raceGenderRows = [
                {
                  race: "White",
                  men: metricById.get("median_weekly_earnings_white_male"),
                  women: metricById.get("median_weekly_earnings_white_female"),
                  total: metricById.get("median_weekly_earnings_white"),
                },
                {
                  race: "Black",
                  men: metricById.get("median_weekly_earnings_black_male"),
                  women: metricById.get("median_weekly_earnings_black_female"),
                  total: metricById.get("median_weekly_earnings_black"),
                },
                {
                  race: "Asian",
                  men: metricById.get("median_weekly_earnings_asian_male"),
                  women: metricById.get("median_weekly_earnings_asian_female"),
                  total: metricById.get("median_weekly_earnings_asian"),
                },
                {
                  race: "Hispanic",
                  men: metricById.get("median_weekly_earnings_hispanic_male"),
                  women: metricById.get("median_weekly_earnings_hispanic_female"),
                  total: metricById.get("median_weekly_earnings_hispanic"),
                },
              ];
              const hasRaceGenderMetrics = raceGenderRows.some(
                (row) => row.men || row.women,
              );
              const allMen = metricById.get("median_weekly_earnings_male");
              const allWomen = metricById.get("median_weekly_earnings_female");
              return (
                <div className="mt-5 grid gap-4 lg:grid-cols-[0.8fr_1.2fr]">
                  <div className="relative overflow-hidden rounded-[1.25rem] border-2 border-black bg-[#a69ad1] p-4 text-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]">
                    <CropMarks className="opacity-25" />
                    <p className="ff-micro text-[10px] font-black opacity-60 leading-none">State gender ratio</p>
                    {male && female ? (
                      <>
                        <p className="ff-pixel mt-2 text-[4.5rem] font-bold leading-none">
                          <Decimals>{((female.observation.value / male.observation.value) * 100).toFixed(1)}%</Decimals>
                        </p>
                        <p className="mt-3 text-xs font-bold opacity-65 leading-tight">
                          Women&apos;s median earnings as a percentage of men&apos;s.
                        </p>
                        <div className="mt-4 grid grid-cols-2 gap-2">
                          <div className="relative overflow-hidden rounded-xl border border-black/15 bg-black/10 p-3">
                            <CropMarks className="opacity-10" />
                            <p className="ff-micro text-[9px] opacity-55 leading-none">Men</p>
                            <p className="ff-pixel mt-1 text-xl font-black leading-none">
                              <Decimals>{formatMoney(male.observation.value)}</Decimals>
                            </p>
                          </div>
                          <div className="relative overflow-hidden rounded-xl border border-black/15 bg-black/10 p-3">
                            <CropMarks className="opacity-10" />
                            <p className="ff-micro text-[9px] opacity-55 leading-none">Women</p>
                            <p className="ff-pixel mt-1 text-xl font-black leading-none">
                              <Decimals>{formatMoney(female.observation.value)}</Decimals>
                            </p>
                          </div>
                        </div>
                      </>
                    ) : (
                      <p className="mt-4 text-xs font-bold opacity-65">No gender wage gap data imported yet.</p>
                    )}
                  </div>
                  <div className="space-y-2">
                    {wageMetrics.length === 0 ? (
                      <p className="text-sm font-mono text-slate-500">No demographic wage data imported yet.</p>
                    ) : (
                      <>
                        <div className="grid grid-cols-[1fr_1fr_1fr] gap-2 px-1 text-[9px] font-black uppercase tracking-[0.16em] text-slate-600">
                          <span>Race</span>
                          <span>Men</span>
                          <span>Women</span>
                        </div>
                        {raceGenderRows.map((row) => (
                          <div key={row.race} className="grid grid-cols-[1fr_1fr_1fr] items-center gap-2 rounded-xl border border-white/5 bg-black/35 px-4 py-3">
                            <span className="text-xs font-bold text-slate-300">{row.race}</span>
                            <span className="ff-pixel text-sm font-black text-[#d2f1ec] leading-none">
                              <Decimals>{row.men ? `${formatMoney(row.men.observation.value)}/wk` : "Not imported"}</Decimals>
                            </span>
                            <span className="ff-pixel text-sm font-black text-[#d2f1ec] leading-none">
                              <Decimals>{row.women ? `${formatMoney(row.women.observation.value)}/wk` : "Not imported"}</Decimals>
                            </span>
                          </div>
                        ))}
                        {!hasRaceGenderMetrics && (
                          <div className="rounded-xl border border-dashed border-white/12 bg-black/20 p-4 text-[11px] font-mono text-slate-500 leading-normal">
                            Race-by-gender BLS CPS series are not imported yet. Current national data has separate gender totals
                            {allMen && allWomen ? ` (${formatMoney(allMen.observation.value)}/wk men, ${formatMoney(allWomen.observation.value)}/wk women)` : ""}
                            {" "}and separate race totals.
                          </div>
                        )}
                      </>
                    )}
                  </div>
                </div>
              );
            })()}
          </div>
        </section>

        <section className="relative overflow-hidden mt-4 rounded-[1.75rem] border-2 border-black bg-white/5 shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] p-5 sm:p-6">
          <CropMarks className="opacity-20" />
          <div className="flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
            <div>
              <p className="ff-micro text-xs font-black text-slate-500">Time series</p>
              <h2 className="ff-sans-wide mt-2 text-2xl font-black tracking-tight text-white">Metric history</h2>
              <p className="mt-1 text-sm font-semibold text-slate-500">
                Last 5 years of monthly/quarterly observations when available.
              </p>
            </div>
          </div>
          <div className="mt-5 grid gap-3.5 lg:grid-cols-3">
            {chartHistories.length === 0 ? (
              <p className="text-sm font-mono text-slate-500 lg:col-span-3">
                No time-series metrics have been imported for this state yet.
              </p>
            ) : (
              chartHistories.map((chart, index) => {
                const series = chart.history.sort((a, b) =>
                  a.observation.date.localeCompare(b.observation.date),
                );
                if (series.length < 2) return null;
                const latest = series[series.length - 1];
                return (
                  <div key={chart.metricId} className="relative overflow-hidden rounded-[1.25rem] border border-black/20 bg-black/35 p-4 shadow-[4px_4px_0px_0px_rgba(0,0,0,0.4)]">
                    <CropMarks className="text-slate-800 opacity-40" />
                    <p className="ff-micro text-[10px] font-black text-slate-500 leading-tight">{chart.label}</p>
                    <p className="ff-pixel mt-3.5 text-3xl font-black leading-none text-white">
                      <Decimals>{formatMetricValue(latest.observation.value, chart.unit)}</Decimals>
                    </p>
                    <BlockHistory series={series} unit={chart.unit} accent={index % 2 === 0 ? "bg-[#d2f1ec]" : "bg-[#f8df1d]"} />
                  </div>
                );
              })
            )}
          </div>
        </section>
      </div>
    </main>
  );
}
