// Design thesis: Bold, blocky Bento Box with solid colors and big typography.
"use client";

import { useParams } from "next/navigation";
import Link from "next/link";
import { useEffect, useState } from "react";
import {
  edgeTypeLabel,
  formatMoney,
  getEntity,
  getEntityAwards,
  getEntityEdges,
} from "@/lib/types";
import type { Award, EntityWithDetails, RelationshipEdge } from "@/lib/types";

export default function EntityPage() {
  const params = useParams<{ id: string }>();
  const [entity, setEntity] = useState<EntityWithDetails | null>(null);
  const [awards, setAwards] = useState<Award[]>([]);
  const [edges, setEdges] = useState<RelationshipEdge[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<"awards" | "edges">("awards");

  useEffect(() => {
    if (!params?.id) return;
    let cancelled = false;

    async function load() {
      try {
        const [entityData, awardsData, edgesData] = await Promise.all([
          getEntity(params.id as string),
          getEntityAwards(params.id as string),
          getEntityEdges(params.id as string),
        ]);
        if (!cancelled) {
          setEntity(entityData);
          setAwards(awardsData);
          setEdges(edgesData);
        }
      } catch (err) {
        if (!cancelled) setError(err instanceof Error ? err.message : "Failed to load");
      } finally {
        if (!cancelled) setLoading(false);
      }
    }

    load();
    return () => {
      cancelled = true;
    };
  }, [params?.id]);

  if (loading) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-bento-dark">
        <div className="text-2xl font-bold animate-pulse text-white">Loading data...</div>
      </main>
    );
  }

  if (error || !entity) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-bento-dark text-white">
        <div className="text-center bg-bento-cardbg p-12 rounded-[2rem] max-w-md">
          <h2 className="text-3xl font-bold mb-4 text-red-400">Error</h2>
          <p className="text-lg opacity-80 mb-8">{error ?? "Entity not found"}</p>
          <Link href="/" className="bg-white text-black px-6 py-3 rounded-full font-bold hover:bg-gray-200 transition">
            Go back
          </Link>
        </div>
      </main>
    );
  }

  const e = entity.entity;
  const totalObligations = awards.reduce(
    (sum, a) => sum + (a.obligation_amount ?? 0),
    0
  );

  return (
    <div className="min-h-screen bg-bento-dark text-white font-sans flex flex-col px-4 py-8">
      <header className="w-full max-w-6xl mx-auto flex items-center justify-between mb-8">
        <Link href="/" className="text-3xl font-bold tracking-tighter hover:opacity-80 transition">FundingFlow</Link>
        <Link href="/" className="bg-white text-black px-4 py-1.5 rounded-full font-medium hover:bg-gray-200 transition">Back to Search</Link>
      </header>

      <main className="flex-1 w-full max-w-6xl mx-auto grid grid-cols-1 md:grid-cols-12 gap-4">
        
        {/* Header Bento Box */}
        <div className="col-span-1 md:col-span-8 bg-bento-purple rounded-[2rem] p-8 md:p-12 text-black flex flex-col justify-between min-h-[300px]">
          <div>
            <span className="bg-black/10 px-4 py-1.5 rounded-full text-sm font-bold uppercase tracking-widest mb-6 inline-block">
              {e.entity_type}
            </span>
            <h1 className="text-4xl md:text-6xl font-bold tracking-tight mb-4 leading-tight">
              {e.display_name}
            </h1>
          </div>
          <div className="text-sm font-bold opacity-60">
            Created on {new Date(e.created_at).toLocaleDateString()}
          </div>
        </div>

        {/* Stats Bento Box */}
        <div className="col-span-1 md:col-span-4 bg-bento-yellow rounded-[2rem] p-8 md:p-12 text-black flex flex-col justify-center min-h-[300px]">
          <div className="mb-8">
            <h3 className="text-sm font-bold opacity-60 uppercase tracking-widest mb-2">Total Obligations</h3>
            <div className="text-4xl md:text-5xl font-bold tracking-tighter">
              {formatMoney(totalObligations)}
            </div>
          </div>
          <div className="flex justify-between mt-auto pt-6 border-t border-black/10">
            <div>
              <h4 className="text-sm font-bold opacity-60 uppercase tracking-widest">Awards</h4>
              <p className="text-2xl font-bold">{awards.length}</p>
            </div>
            <div>
              <h4 className="text-sm font-bold opacity-60 uppercase tracking-widest">Edges</h4>
              <p className="text-2xl font-bold">{edges.length}</p>
            </div>
          </div>
        </div>

        {/* Identifiers Bento Box */}
        {entity.identifiers.length > 0 && (
          <div className="col-span-1 md:col-span-12 bg-bento-olive rounded-[2rem] p-8 md:p-12 text-black">
            <h3 className="text-xl font-bold mb-6">Identifiers</h3>
            <div className="grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 gap-4">
              {entity.identifiers.map((id) => (
                <div key={id.id} className="bg-black/10 p-4 rounded-xl">
                  <div className="font-mono text-lg font-bold">{id.identifier_value}</div>
                  <div className="text-xs font-bold opacity-70 mt-1 uppercase">
                    {id.source} / {id.identifier_type}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Tabs Control */}
        <div className="col-span-1 md:col-span-12 flex gap-2 my-4">
          <button
            type="button"
            onClick={() => setTab("awards")}
            className={`px-6 py-3 rounded-full font-bold transition-colors ${
              tab === "awards" ? "bg-white text-black" : "bg-bento-cardbg text-white hover:bg-white/10"
            }`}
          >
            Awards
          </button>
          <button
            type="button"
            onClick={() => setTab("edges")}
            className={`px-6 py-3 rounded-full font-bold transition-colors ${
              tab === "edges" ? "bg-white text-black" : "bg-bento-cardbg text-white hover:bg-white/10"
            }`}
          >
            Relationships
          </button>
        </div>

        {/* Content Lists */}
        <div className="col-span-1 md:col-span-12 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {tab === "awards" && (
            awards.length === 0 ? (
              <div className="col-span-1 md:col-span-3 bg-bento-cardbg rounded-[2rem] p-12 text-center text-xl font-medium text-gray-400">
                No awards found.
              </div>
            ) : (
              awards.map((award, i) => {
                const bgs = ["bg-bento-cardbg text-white", "bg-[#e5e7eb] text-black", "bg-bento-blue text-black"];
                const colorClass = bgs[i % bgs.length];

                return (
                  <div key={award.id} className={`col-span-1 rounded-[2rem] p-8 flex flex-col justify-between min-h-[250px] ${colorClass}`}>
                    <div>
                      <div className="flex justify-between items-start mb-4">
                        <span className={`px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider ${colorClass.includes('text-white') ? 'bg-white/20' : 'bg-black/10'}`}>
                          {award.award_type ?? "Award"}
                        </span>
                      </div>
                      <h4 className="text-3xl font-bold tracking-tight mb-2">
                        {formatMoney(award.obligation_amount)}
                      </h4>
                      {award.description && (
                        <p className={`text-sm font-medium mt-4 line-clamp-3 ${colorClass.includes('text-white') ? 'opacity-80' : 'opacity-70'}`}>
                          {award.description}
                        </p>
                      )}
                    </div>
                    <div className="mt-8 pt-4 border-t border-current/10 flex flex-col gap-1 text-xs font-bold uppercase tracking-wider opacity-70">
                      {award.period_start && <span>Start: {award.period_start}</span>}
                      {award.place_of_performance && <span>Loc: {award.place_of_performance}</span>}
                      <span>ID: {award.generated_unique_award_id}</span>
                    </div>
                  </div>
                );
              })
            )
          )}

          {tab === "edges" && (
            edges.length === 0 ? (
              <div className="col-span-1 md:col-span-3 bg-bento-cardbg rounded-[2rem] p-12 text-center text-xl font-medium text-gray-400">
                No relationships recorded.
              </div>
            ) : (
              edges.map((edge, i) => {
                const bgs = ["bg-bento-cardbg text-white", "bg-bento-orange text-black", "bg-[#e5e7eb] text-black"];
                const colorClass = bgs[i % bgs.length];

                return (
                  <div key={edge.id} className={`col-span-1 rounded-[2rem] p-8 flex flex-col justify-between min-h-[250px] ${colorClass}`}>
                    <div>
                      <div className="flex justify-between items-start mb-4">
                        <span className={`px-3 py-1 rounded-full text-xs font-bold uppercase tracking-wider ${colorClass.includes('text-white') ? 'bg-white/20' : 'bg-black/10'}`}>
                          {edgeTypeLabel(edge.edge_type)}
                        </span>
                      </div>
                      {edge.amount != null && (
                        <h4 className="text-3xl font-bold tracking-tight mb-2">
                          {formatMoney(edge.amount)}
                        </h4>
                      )}
                      {edge.description && (
                        <p className={`text-sm font-medium mt-4 line-clamp-3 ${colorClass.includes('text-white') ? 'opacity-80' : 'opacity-70'}`}>
                          {edge.description}
                        </p>
                      )}
                    </div>
                    <div className="mt-8 pt-4 border-t border-current/10 flex flex-col gap-1 text-xs font-bold uppercase tracking-wider opacity-70">
                      {edge.started_on && <span>Start: {edge.started_on}</span>}
                      <span>Source: {edge.source}</span>
                      {edge.confidence != null && <span>Conf: {(edge.confidence * 100).toFixed(0)}%</span>}
                    </div>
                  </div>
                );
              })
            )
          )}
        </div>
      </main>
    </div>
  );
}
