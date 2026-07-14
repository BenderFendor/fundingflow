"use client";

import { useState, type FormEvent } from "react";
import { useRouter } from "next/navigation";
import { formatMoney, unifiedSearch } from "@/lib/types";
import type { EntitySummary, Geo } from "@/lib/types";
import { CropMarks, Decimals } from "@/components/ui";

export function HomeSearch() {
  const router = useRouter();
  const [query, setQuery] = useState("");
  const [entities, setEntities] = useState<EntitySummary[] | null>(null);
  const [geos, setGeos] = useState<Geo[] | null>(null);
  const [entityTotal, setEntityTotal] = useState(0);
  const [geoTotal, setGeoTotal] = useState(0);
  const [loading, setLoading] = useState(false);

  async function handleSearch(e: FormEvent) {
    e.preventDefault();
    if (!query.trim()) return;
    setLoading(true);
    try {
      const data = await unifiedSearch(query.trim());
      setEntities(data.entities);
      setEntityTotal(data.entity_total);
      setGeos(data.geos);
      setGeoTotal(data.geo_total);
    } catch {
      setEntities([]);
      setEntityTotal(0);
      setGeos([]);
      setGeoTotal(0);
    } finally {
      setLoading(false);
    }
  }

  const hasResults = entities !== null || geos !== null;

  return (
    <>
      <form onSubmit={handleSearch} className="w-full max-w-2xl relative">
        <div className="relative flex flex-col overflow-hidden rounded-[1.25rem] border-2 border-black bg-black shadow-2xl sm:flex-row sm:items-center">
          <input
            id="global-search"
            name="q"
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search Boeing, Elon, California, DUNS..."
            className="ff-focus-ring w-full bg-transparent px-5 py-5 text-lg font-semibold text-white placeholder:text-slate-500 focus:outline-none sm:px-6"
          />
          <button type="submit" className="ff-focus-ring bg-[#f7df1e] px-6 py-4 text-sm font-black uppercase tracking-[0.18em] text-black transition-colors hover:bg-[#ffe94a] sm:py-5">
            {loading ? "Searching" : "Search"}
          </button>
        </div>
      </form>

      {loading && (
        <div className="mt-4 flex justify-start">
          <div className="rounded-full bg-black/10 px-4 py-2 text-sm font-black uppercase tracking-[0.18em] text-black/70">
            Loading data
          </div>
        </div>
      )}

      {hasResults && !loading && (
        <div className="mt-8 w-full max-w-6xl">
          <div className="mb-8 flex flex-wrap gap-4 border-b border-white/10 pb-4">
            <span className="font-mono text-[9px] font-black uppercase tracking-widest text-[#DFFF00]">
              {geoTotal + entityTotal} matches
            </span>
            <span className="font-mono text-[9px] font-medium uppercase tracking-widest text-slate-500">
              {geoTotal} locations
            </span>
            <span className="font-mono text-[9px] font-medium uppercase tracking-widest text-slate-500">
              {entityTotal} entities
            </span>
          </div>

          {geos && geos.length > 0 && (
            <div className="mb-12">
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-mono text-[10px] uppercase tracking-widest text-slate-500">Locations</h3>
              </div>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-[1px] bg-white/10 border border-white/10">
                {geos.map((geo) => {
                  const href = geo.geo_type === "nation"
                    ? "/"
                    : geo.geo_type === "state"
                      ? `/states/${geo.geo_id}`
                      : `/counties/${geo.state_code ?? ""}/${geo.county_code ?? ""}`;
                  return (
                    <button
                      key={geo.geo_id}
                      type="button"
                      onClick={() => router.push(href)}
                      className="ff-focus-ring group relative overflow-hidden bg-black p-6 text-left hover:bg-white/5 transition-colors"
                    >
                      <CropMarks className="opacity-20" />
                      <span className="font-mono text-[9px] font-black uppercase tracking-widest text-slate-500 leading-none">
                        {geo.geo_type}
                      </span>
                      <h4 className="ff-sans-wide mt-3 text-2xl font-black tracking-tighter text-white leading-tight group-hover:text-[#DFFF00] transition-colors">{geo.name}</h4>
                      <p className="mt-4 font-mono text-[9px] font-medium uppercase tracking-widest text-slate-600 leading-none">{geo.geo_id}</p>
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {entities && entities.length > 0 && (
            <div>
              <div className="flex items-center justify-between mb-4">
                <h3 className="font-mono text-[10px] uppercase tracking-widest text-slate-500">Organizations and people</h3>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-[1px] bg-white/10 border border-white/10">
                {entities.map((entity, i) => {
                  return (
                    <button
                      key={entity.id}
                      type="button"
                      onClick={() => router.push(`/entities/${entity.id}`)}
                      className="ff-focus-ring group relative overflow-hidden col-span-1 flex min-h-[260px] flex-col justify-between bg-black p-8 text-left hover:bg-white/5 transition-colors"
                    >
                      <CropMarks className="opacity-20" />
                      <div>
                        <div className="flex justify-between items-start mb-6">
                          <span className="font-mono text-[9px] font-black uppercase tracking-widest text-[#DFFF00] bg-[#DFFF00]/10 px-3 py-1.5 rounded-full border border-[#DFFF00]/20">
                            {entity.entity_type}
                          </span>
                        </div>
                        <h4 className="ff-sans-wide text-3xl font-black leading-tight tracking-tighter text-white group-hover:text-[#DFFF00] transition-colors">{entity.display_name}</h4>
                      </div>
                      <div className="mt-8 pt-6 border-t border-white/10">
                        <div className="mb-5 flex gap-8">
                          <div className="flex flex-col">
                            <span className="font-mono text-[9px] uppercase tracking-widest text-slate-500 leading-none">IDs</span>
                            <span className="ff-pixel text-2xl font-bold text-white leading-none mt-2">{entity.identifier_count}</span>
                          </div>
                          <div className="flex flex-col">
                            <span className="font-mono text-[9px] uppercase tracking-widest text-slate-500 leading-none">Awards</span>
                            <span className="ff-pixel text-2xl font-bold text-white leading-none mt-2">{entity.award_count}</span>
                          </div>
                        </div>
                        {entity.total_obligations != null && (
                          <div className="flex flex-col">
                            <span className="font-mono text-[9px] uppercase tracking-widest text-slate-500 leading-none mb-2">Obligations</span>
                            <span className="ff-pixel text-3xl font-bold text-[#DFFF00] leading-none">
                              <Decimals>{formatMoney(entity.total_obligations)}</Decimals>
                            </span>
                          </div>
                        )}
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>
          )}

          {entities && entities.length === 0 && geos && geos.length === 0 && (
            <div className="border border-dashed border-white/20 p-10 text-center">
              <p className="font-mono text-[10px] uppercase tracking-widest text-slate-500">No entities or locations matched your query.</p>
            </div>
          )}
        </div>
      )}
    </>
  );
}
