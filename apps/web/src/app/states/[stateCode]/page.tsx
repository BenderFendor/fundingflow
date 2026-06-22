import Link from "next/link";
import {
  derivedMetricLabel,
  formatMetricValue,
  formatMoney,
  getNationalPulse,
  getStateProfile,
} from "@/lib/types";

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

export default async function StateProfilePage({ params }: StatePageProps) {
  const { stateCode } = await params;
  const [profile, nationalPulse] = await Promise.all([
    getStateProfile(stateCode.toUpperCase()),
    getNationalPulse(),
  ]);
  const publicMoneyMetric = profile.latest_metrics.find(
    (item) => item.metric.metric_id === "federal_contract_obligations"
  );
  const displayedObligations =
    profile.public_money.total_obligations || publicMoneyMetric?.observation.value || 0;

  const metricsByCategory = new Map(
    sections.map((section) => [
      section.key,
      profile.latest_metrics.filter((item) => item.metric.category === section.key),
    ])
  );
  const nationalFoodMetrics = nationalPulse.latest_metrics.filter((item) =>
    [
      "food_at_home_cpi_yoy",
      "food_away_from_home_cpi_yoy",
      "egg_price",
      "milk_price",
      "white_bread_price",
      "ground_beef_price",
      "chicken_breast_price",
      "banana_price",
    ].includes(item.metric.metric_id)
  );
  const nationalFoodDerivedMetrics = (nationalPulse.derived_metrics ?? []).filter((metric) =>
    ["food_basket_cost", "food_basket_as_percent_weekly_wage"].includes(metric.metric_id)
  );

  return (
    <main className="mx-auto min-h-screen max-w-6xl px-4 py-8 text-slate-100">
      <header className="flex flex-col gap-6 border-b border-white/10 pb-8 md:flex-row md:items-end md:justify-between">
        <div>
          <Link href="/" className="text-sm text-slate-400 hover:text-slate-200">
            &larr; Back to search
          </Link>
          <p className="mt-8 text-sm uppercase tracking-[0.2em] text-emerald-300">
            State Economic Conditions
          </p>
          <h1 className="mt-2 text-5xl font-black tracking-tight">{profile.geo.name}</h1>
          <p className="mt-4 max-w-3xl text-lg text-slate-300">
            Public money, labor markets, rent pressure, food and energy costs,
            and state economic structure in one source-backed profile.
          </p>
        </div>
        <div className="rounded-lg border border-white/10 bg-white/5 p-4">
          <p className="text-sm text-slate-400">Federal obligations in profile</p>
          <p className="mt-1 text-3xl font-black">
            {formatMoney(displayedObligations)}
          </p>
          <p className="mt-1 text-sm text-slate-400">
            {profile.public_money.award_count.toLocaleString()} linked award records
          </p>
        </div>
      </header>

      <section className="mt-8 grid gap-3 md:grid-cols-3">
        {profile.derived_metrics.length === 0 ? (
          <div className="rounded-lg border border-dashed border-white/15 p-5 text-slate-400 md:col-span-3">
            Derived pressure metrics are not available yet for this geography.
          </div>
        ) : (
          profile.derived_metrics.map((metric) => (
            <article key={metric.metric_id} className="rounded-lg bg-emerald-300 p-5 text-black">
              <p className="text-sm font-bold uppercase tracking-wide opacity-70">
                {derivedMetricLabel(metric.metric_id)}
              </p>
              <p className="mt-2 text-3xl font-black">
                {metric.metric_id.includes("intensity")
                  ? `${(metric.value * 100).toFixed(3)}%`
                  : metric.value.toFixed(1)}
              </p>
              <p className="mt-3 text-xs font-medium opacity-70">
                Formula: {metric.formula}. Vintage: {metric.vintage_date}.
              </p>
            </article>
          ))
        )}
      </section>

      <section className="mt-8 rounded-lg border border-white/10 bg-white/[0.03] p-5">
        <div className="flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
          <div>
            <h2 className="text-xl font-bold">Food Price Basket</h2>
            <p className="mt-1 text-sm text-slate-500">
              National BLS CPI and U.S. city average prices. Use as pressure context, not state grocery data.
            </p>
          </div>
          <p className="text-xs uppercase tracking-[0.18em] text-slate-500">
            Scope: {nationalPulse.geo.name}
          </p>
        </div>
        {nationalFoodMetrics.length === 0 ? (
          <p className="mt-4 text-sm text-slate-500">
            No national food price metrics have been imported yet.
          </p>
        ) : (
          <>
            {nationalFoodDerivedMetrics.length > 0 && (
              <div className="mt-4 grid gap-3 md:grid-cols-2">
                {nationalFoodDerivedMetrics.map((metric) => (
                  <div key={metric.metric_id} className="rounded-lg bg-emerald-300 p-4 text-black">
                    <p className="text-sm font-bold uppercase tracking-wide opacity-70">
                      {derivedMetricLabel(metric.metric_id)}
                    </p>
                    <p className="mt-2 text-3xl font-black">
                      {metric.metric_id.includes("percent")
                        ? `${metric.value.toFixed(2)}%`
                        : formatMoney(metric.value)}
                    </p>
                    <p className="mt-2 text-xs font-medium opacity-70">
                      Formula: {metric.formula}. Vintage: {metric.vintage_date}.
                    </p>
                  </div>
                ))}
              </div>
            )}
            <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
              {nationalFoodMetrics.map((item) => (
                <div key={item.metric.metric_id} className="rounded-lg bg-black/20 p-4">
                  <p className="text-sm font-semibold text-slate-200">{item.metric.name}</p>
                  <p className="mt-2 text-2xl font-black">
                    {formatMetricValue(item.observation.value, item.metric.unit)}
                  </p>
                  <p className="mt-2 text-xs text-slate-500">
                    {item.observation.date} / {item.observation.source_series_id}
                  </p>
                </div>
              ))}
            </div>
          </>
        )}
      </section>

      <section className="mt-8 grid gap-4 lg:grid-cols-2">
        {sections.map((section) => {
          const metrics = metricsByCategory.get(section.key) ?? [];
          return (
            <article key={section.key} className="rounded-lg border border-white/10 bg-white/[0.03] p-5">
              <h2 className="text-xl font-bold">{section.title}</h2>
              {metrics.length === 0 ? (
                <p className="mt-4 text-sm text-slate-500">
                  No sourced metric has been imported for this section yet.
                </p>
              ) : (
                <div className="mt-4 divide-y divide-white/10">
                  {metrics.map((item) => (
                    <div key={item.metric.metric_id} className="py-3">
                      <div className="flex items-start justify-between gap-4">
                        <div>
                          <p className="font-semibold">{item.metric.name}</p>
                          <p className="mt-1 text-xs text-slate-500">
                            {item.source.agency} / {item.source.dataset}
                          </p>
                        </div>
                        <p className="shrink-0 text-xl font-black">
                          {formatMetricValue(item.observation.value, item.metric.unit)}
                        </p>
                      </div>
                      <p className="mt-2 text-xs text-slate-500">
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

      <section className="mt-8 rounded-lg border border-white/10 bg-white/[0.03] p-5">
        <h2 className="text-xl font-bold">Top Public-Money Recipients</h2>
        {profile.public_money.top_recipients.length === 0 ? (
          <p className="mt-4 text-sm text-slate-500">
            No federal award geography has been linked to this state yet.
          </p>
        ) : (
          <div className="mt-4 divide-y divide-white/10">
            {profile.public_money.top_recipients.map((recipient) => (
              <div key={`${recipient.entity_id}-${recipient.display_name}`} className="flex items-center justify-between gap-4 py-3">
                <div>
                  <p className="font-semibold">{recipient.display_name}</p>
                  <p className="text-xs text-slate-500">
                    {recipient.award_count.toLocaleString()} award records
                  </p>
                </div>
                <p className="font-black">{formatMoney(recipient.total_obligations)}</p>
              </div>
            ))}
          </div>
        )}
      </section>
    </main>
  );
}
