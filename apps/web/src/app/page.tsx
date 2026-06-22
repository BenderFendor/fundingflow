// Design thesis: Bold, blocky Bento Box with solid colors and big typography.
"use client";

import { useState, type FormEvent } from "react";
import Link from "next/link";
import { formatMoney, searchEntities } from "@/lib/types";
import type { EntitySummary } from "@/lib/types";
import { useRouter } from "next/navigation";

export default function HomePage() {
  const router = useRouter();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<EntitySummary[] | null>(null);
  const [total, setTotal] = useState(0);
  const [loading, setLoading] = useState(false);

  async function handleSearch(e: FormEvent) {
    e.preventDefault();
    if (!query.trim()) return;
    setLoading(true);
    try {
      const data = await searchEntities(query.trim());
      setResults(data.entities);
      setTotal(data.total);
    } catch {
      setResults([]);
      setTotal(0);
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="min-h-screen bg-bento-dark text-white font-sans relative flex flex-col px-4 py-8">
      <header className="w-full max-w-6xl mx-auto flex items-center justify-between mb-8">
        <h1 className="text-3xl font-bold tracking-tighter">FundingFlow</h1>
        <nav className="flex items-center gap-4 text-sm font-medium">
          <Link href="/states/PA" className="border border-white/20 px-4 py-1.5 rounded-full hover:bg-white/10 transition">Pennsylvania</Link>
          <Link href="/methodology" className="bg-white text-black px-4 py-1.5 rounded-full hover:bg-gray-200 transition">Methodology</Link>
          <Link href="/data" className="border border-white/20 px-4 py-1.5 rounded-full hover:bg-white/10 transition">Data Lineage</Link>
        </nav>
      </header>

      <main className="flex-1 w-full max-w-6xl mx-auto grid grid-cols-1 md:grid-cols-12 gap-4">
        
        {/* Search Bento Box */}
        <div className="col-span-1 md:col-span-12 bg-bento-purple rounded-[2rem] p-8 md:p-12 text-black flex flex-col justify-center min-h-[320px]">
          <h2 className="text-4xl md:text-6xl font-bold tracking-tight mb-4 leading-tight">
            Trace the flow<br />of federal power.
          </h2>
          <p className="text-lg md:text-xl font-medium opacity-80 max-w-2xl mb-8">
            Public-record graph for federal spending, lobbying, campaign finance,
            corporate disclosures, labor markets, rent pressure, food costs, and energy prices.
          </p>
          
          <form onSubmit={handleSearch} className="w-full max-w-2xl relative">
            <div className="relative flex items-center bg-black rounded-2xl overflow-hidden shadow-xl">
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search organizations, people, or identifiers..."
                className="w-full bg-transparent py-5 px-6 text-xl text-white placeholder:text-gray-400 focus:outline-none"
              />
              <button type="submit" className="px-8 py-5 text-lg font-bold text-black bg-bento-yellow hover:bg-[#e6c700] transition-colors">
                Search
              </button>
            </div>
          </form>
        </div>

        <div className="col-span-1 md:col-span-12 grid grid-cols-1 gap-4 md:grid-cols-4">
          {[
            ["PA", "Pennsylvania"],
            ["CA", "California"],
            ["TX", "Texas"],
            ["NY", "New York"],
          ].map(([code, name]) => (
            <Link
              key={code}
              href={`/states/${code}`}
              className="rounded-[1.25rem] border border-white/10 bg-white/[0.04] p-5 transition hover:bg-white/10"
            >
              <span className="text-xs font-bold uppercase tracking-[0.2em] text-emerald-300">
                State Profile
              </span>
              <h3 className="mt-2 text-2xl font-black">{name}</h3>
              <p className="mt-2 text-sm text-slate-400">
                Labor, rent, food, energy, and public-money conditions.
              </p>
            </Link>
          ))}
        </div>

        {/* Results Area */}
        {loading && (
          <div className="col-span-1 md:col-span-12 bg-bento-cardbg rounded-[2rem] p-12 flex justify-center items-center">
            <div className="text-2xl font-bold animate-pulse text-white">Loading data...</div>
          </div>
        )}

        {results !== null && !loading && (
          <div className="col-span-1 md:col-span-12 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div className="col-span-1 md:col-span-2 lg:col-span-3 mb-2 flex items-center justify-between">
              <h3 className="text-2xl font-bold text-white">Search Results</h3>
              <span className="text-lg font-medium bg-white text-black px-4 py-1 rounded-full">{total} Found</span>
            </div>

            {results.length === 0 ? (
              <div className="col-span-1 md:col-span-3 bg-bento-cardbg rounded-[2rem] p-12 text-center">
                <p className="text-xl font-medium text-gray-400">No entities matched your query.</p>
              </div>
            ) : (
              results.map((entity, i) => {
                // cycle through some colors for the cards
                const bgs = ["bg-bento-cardbg text-white", "bg-bento-olive text-black", "bg-[#e5e7eb] text-black", "bg-bento-orange text-black"];
                const colorClass = bgs[i % bgs.length];

                return (
                  <button
                    key={entity.id}
                    onClick={() => router.push(`/entities/${entity.id}`)}
                    className={`col-span-1 rounded-[2rem] p-6 text-left transition-transform hover:-translate-y-1 hover:shadow-2xl flex flex-col justify-between min-h-[220px] ${colorClass}`}
                  >
                    <div>
                      <div className="flex justify-between items-start mb-4">
                        <span className={`px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider ${colorClass.includes('text-white') ? 'bg-white/20' : 'bg-black/10'}`}>
                          {entity.entity_type}
                        </span>
                      </div>
                      <h4 className="text-2xl font-bold leading-tight mb-2">{entity.display_name}</h4>
                    </div>
                    
                    <div>
                      <div className="flex gap-4 mb-4">
                        <div className="flex flex-col">
                          <span className={`text-sm font-bold opacity-60`}>IDs</span>
                          <span className="text-xl font-bold">{entity.identifier_count}</span>
                        </div>
                        <div className="flex flex-col">
                          <span className={`text-sm font-bold opacity-60`}>Awards</span>
                          <span className="text-xl font-bold">{entity.award_count}</span>
                        </div>
                      </div>
                      
                      {entity.total_obligations != null && (
                        <div className="flex flex-col mt-auto pt-4 border-t border-black/10">
                          <span className={`text-sm font-bold opacity-60`}>Obligations</span>
                          <span className="text-2xl font-bold tracking-tight">
                            {formatMoney(entity.total_obligations)}
                          </span>
                        </div>
                      )}
                    </div>
                  </button>
                );
              })
            )}
          </div>
        )}
      </main>
    </div>
  );
}
