"use client";

import Link from "next/link";
import { useState, type FormEvent } from "react";
import { formatMoney } from "@/lib/types";
import type { EntitySummary, Geo } from "@/lib/types";
import { unifiedSearchClient } from "@/lib/client-api";
import { CropMarks, DataBadge, Decimals } from "@/components/ui";

const suggestedQueries = ["Lockheed Martin", "California", "Boeing", "Elon Musk"];

export function HomeSearch({ initialQuery = "" }: { initialQuery?: string }) {
  const [query, setQuery] = useState(initialQuery);
  const [entities, setEntities] = useState<EntitySummary[] | null>(null);
  const [geos, setGeos] = useState<Geo[] | null>(null);
  const [entityTotal, setEntityTotal] = useState(0);
  const [geoTotal, setGeoTotal] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function runSearch(searchQuery: string) {
    const normalizedQuery = searchQuery.trim();
    if (!normalizedQuery || loading) return;

    setQuery(normalizedQuery);
    setLoading(true);
    setError(null);

    try {
      const data = await unifiedSearchClient(normalizedQuery);
      setEntities(data.entities);
      setEntityTotal(data.entity_total);
      setGeos(data.geos);
      setGeoTotal(data.geo_total);
    } catch (searchError) {
      setEntities([]);
      setEntityTotal(0);
      setGeos([]);
      setGeoTotal(0);
      setError(searchError instanceof Error ? searchError.message : "Search is temporarily unavailable.");
    } finally {
      setLoading(false);
    }
  }

  function handleSearch(event: FormEvent) {
    event.preventDefault();
    void runSearch(query);
  }

  const hasResults = entities !== null || geos !== null;
  const totalMatches = entityTotal + geoTotal;

  return (
    <div className="w-full">
      <form onSubmit={handleSearch} className="relative" role="search" aria-busy={loading}>
        <label htmlFor="global-search" className="ff-micro mb-3 block text-[9px] font-black text-black/50">
          Search the public-record graph
        </label>
        <div className="grid overflow-hidden border-2 border-black bg-black shadow-[5px_5px_0_rgba(0,0,0,0.18)] sm:grid-cols-[1fr_auto]">
          <div className="relative flex min-w-0 items-center">
            <span aria-hidden="true" className="ml-5 font-mono text-sm text-[#dfff00]">/</span>
            <input
              id="global-search"
              name="q"
              type="search"
              autoComplete="off"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              placeholder="Organization, person, state, UEI, CIK, FEC ID..."
              className="ff-focus-ring min-w-0 flex-1 bg-transparent px-4 py-5 text-base font-semibold text-white placeholder:text-slate-600 focus:outline-none sm:py-6 sm:text-lg"
            />
            <kbd className="mr-4 hidden border border-white/10 bg-white/5 px-2 py-1 font-mono text-[8px] text-slate-600 md:block">ENTER</kbd>
          </div>
          <button
            type="submit"
            disabled={loading || !query.trim()}
            className="ff-focus-ring ff-fluid min-w-36 border-t-2 border-black bg-[#dfff00] px-6 py-4 font-mono text-[10px] font-black uppercase tracking-[0.16em] text-black hover:bg-[#efff68] disabled:cursor-not-allowed disabled:bg-[#798640] disabled:text-black/60 sm:border-l-2 sm:border-t-0"
          >
            {loading ? "Querying" : "Run search"}
          </button>
        </div>

        <div className="mt-4 flex flex-wrap items-center gap-2">
          <span className="font-mono text-[8px] uppercase tracking-[0.14em] text-black/40">Try</span>
          {suggestedQueries.map((suggestion) => (
            <button
              key={suggestion}
              type="button"
              onClick={() => void runSearch(suggestion)}
              className="ff-focus-ring ff-fluid border border-black/10 bg-black/[0.045] px-3 py-1.5 font-mono text-[9px] font-bold text-black/55 hover:border-black/25 hover:bg-black/10 hover:text-black"
            >
              {suggestion}
            </button>
          ))}
        </div>
      </form>

      <div aria-live="polite" className="sr-only">
        {loading ? "Searching FundingFlow" : hasResults ? `${totalMatches} results found` : ""}
      </div>

      {error ? (
        <div className="mt-5 border border-[#ee744f]/35 bg-[#ee744f]/10 p-4 text-sm text-[#ffc3b1]">
          <p className="font-semibold">Search could not reach the data service.</p>
          <p className="mt-1 text-xs text-[#ffc3b1]/70">{error}</p>
        </div>
      ) : null}

      {hasResults && !loading ? (
        <section className="ff-terminal ff-enter mt-8 overflow-hidden text-white">
          <div className="flex flex-col gap-4 border-b border-white/10 px-5 py-5 sm:flex-row sm:items-center sm:justify-between sm:px-6">
            <div>
              <p className="ff-kicker text-[#dfff00]">Query results</p>
              <h2 className="ff-sans-wide mt-3 text-2xl font-black text-white">
                {totalMatches.toLocaleString()} matches for “{query}”
              </h2>
            </div>
            <div className="flex gap-2">
              <DataBadge tone="acid">{entityTotal} entities</DataBadge>
              <DataBadge>{geoTotal} geographies</DataBadge>
            </div>
          </div>

          {geos && geos.length > 0 ? (
            <div className="border-b border-white/10">
              <div className="flex items-center justify-between px-5 py-4 sm:px-6">
                <h3 className="ff-micro text-[9px] font-black text-slate-500">Geography matches</h3>
                <span className="font-mono text-[8px] uppercase tracking-wider text-slate-700">Open profile</span>
              </div>
              <div className="grid border-t border-white/10 sm:grid-cols-2 lg:grid-cols-4">
                {geos.map((geo, index) => {
                  const href = geo.geo_type === "nation"
                    ? "/"
                    : geo.geo_type === "state"
                      ? `/states/${geo.geo_id}`
                      : `/counties/${geo.state_code ?? ""}/${geo.county_code ?? ""}`;
                  return (
                    <Link
                      key={geo.geo_id}
                      href={href}
                      className="ff-focus-ring ff-interactive group relative min-h-40 overflow-hidden border-r border-b border-white/10 bg-black/25 p-5 hover:bg-white/[0.045]"
                    >
                      <CropMarks className="text-white/15" />
                      <div className="flex items-start justify-between">
                        <DataBadge>{geo.geo_type}</DataBadge>
                        <span className="font-mono text-[9px] text-slate-700">0{index + 1}</span>
                      </div>
                      <h4 className="ff-sans-wide mt-8 text-2xl font-black leading-tight text-white group-hover:text-[#dfff00]">
                        {geo.name}
                      </h4>
                      <p className="mt-3 font-mono text-[9px] uppercase tracking-widest text-slate-600">{geo.geo_id}</p>
                    </Link>
                  );
                })}
              </div>
            </div>
          ) : null}

          {entities && entities.length > 0 ? (
            <div>
              <div className="flex items-center justify-between px-5 py-4 sm:px-6">
                <h3 className="ff-micro text-[9px] font-black text-slate-500">Entity matches</h3>
                <span className="font-mono text-[8px] uppercase tracking-wider text-slate-700">Ranked by search relevance</span>
              </div>
              <div className="divide-y divide-white/10 border-t border-white/10">
                {entities.map((entity, index) => (
                  <Link
                    key={entity.id}
                    href={`/entities/${entity.id}`}
                    className="ff-focus-ring group grid gap-5 px-5 py-5 transition-colors hover:bg-white/[0.045] sm:grid-cols-[3.5rem_minmax(0,1fr)_auto] sm:items-center sm:px-6"
                  >
                    <span className="ff-pixel text-xl font-black text-white/18">{String(index + 1).padStart(2, "0")}</span>
                    <div className="min-w-0">
                      <div className="flex flex-wrap items-center gap-2">
                        <h4 className="ff-sans-wide truncate text-xl font-black text-white group-hover:text-[#dfff00]">{entity.display_name}</h4>
                        <DataBadge tone="acid">{entity.entity_type}</DataBadge>
                      </div>
                      <p className="mt-2 font-mono text-[9px] uppercase tracking-wider text-slate-600">
                        {entity.identifier_count} identifiers / {entity.award_count} awards
                      </p>
                    </div>
                    <div className="sm:text-right">
                      <p className="ff-micro text-[8px] font-black text-slate-700">Linked obligations</p>
                      <p className="ff-pixel mt-2 text-2xl font-black text-white">
                        <Decimals>{formatMoney(entity.total_obligations)}</Decimals>
                      </p>
                    </div>
                  </Link>
                ))}
              </div>
            </div>
          ) : null}

          {entities?.length === 0 && geos?.length === 0 ? (
            <div className="px-6 py-12 text-center">
              <p className="ff-sans-wide text-2xl font-black text-white">No matching records</p>
              <p className="mx-auto mt-3 max-w-xl text-sm leading-6 text-slate-500">
                Try a shorter legal name, a state name, or a public identifier such as a UEI, CIK, FEC committee ID, or LDA identifier.
              </p>
            </div>
          ) : null}
        </section>
      ) : null}
    </div>
  );
}
