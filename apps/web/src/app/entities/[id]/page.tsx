"use client";

import { useParams } from "next/navigation";
import Link from "next/link";
import { useEffect, useState } from "react";
import {
  derivedMetricLabel,
  edgeTypeLabel,
  formatMetricValue,
  formatMoney,
  getEntity,
  getEntityAwards,
  getEntityEdges,
  getEntityEconomicContext,
  getEntityLobbying,
  getEntityContributions,
} from "@/lib/types";
import { CropMarks, Decimals } from "@/components/ui";
import type {
  Award,
  EntityWithDetails,
  LobbyingFiling,
  RelationshipEdge,
  StateProfile,
  EntityContributionsResponse,
} from "@/lib/types";

type Tab = "awards" | "lobbying" | "edges" | "economic" | "finance";

export default function EntityPage() {
  const params = useParams<{ id: string }>();
  const [entity, setEntity] = useState<EntityWithDetails | null>(null);
  const [awards, setAwards] = useState<Award[]>([]);
  const [lobbying, setLobbying] = useState<LobbyingFiling[]>([]);
  const [edges, setEdges] = useState<RelationshipEdge[]>([]);
  const [economicContext, setEconomicContext] = useState<StateProfile[]>([]);
  const [contributions, setContributions] = useState<EntityContributionsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("awards");

  useEffect(() => {
    if (!params?.id) return;
    let cancelled = false;

    async function load() {
      try {
        const [entityData, awardsData, edgesData, lobbyingData, econData, contribData] =
          await Promise.all([
            getEntity(params.id as string),
            getEntityAwards(params.id as string),
            getEntityEdges(params.id as string),
            getEntityLobbying(params.id as string),
            getEntityEconomicContext(params.id as string),
            getEntityContributions(params.id as string),
          ]);
        if (!cancelled) {
          setEntity(entityData);
          setAwards(awardsData);
          setLobbying(lobbyingData);
          setEdges(edgesData);
          setEconomicContext(econData ?? []);
          setContributions(contribData);
        }
      } catch (err) {
        if (!cancelled)
          setError(err instanceof Error ? err.message : "Failed to load");
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
      <main className="flex min-h-screen items-center justify-center bg-[#090909]">
        <div className="text-2xl font-bold animate-pulse text-white">
          Loading data...
        </div>
      </main>
    );
  }

  if (error || !entity) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-[#090909] text-white">
        <div className="text-center bg-[#111] p-12 rounded-[2rem] max-w-md">
          <h2 className="text-3xl font-bold mb-4 text-red-400">Error</h2>
          <p className="text-lg opacity-80 mb-8">
            {error ?? "Entity not found"}
          </p>
          <Link
            href="/"
            className="bg-white text-black px-6 py-3 rounded-full font-bold hover:bg-gray-200 transition"
          >
            Go back
          </Link>
        </div>
      </main>
    );
  }

  const e = entity.entity;
  const totalObligations = awards.reduce(
    (sum, a) => sum + (a.obligation_amount ?? 0),
    0,
  );
  const totalLobbyingAmount = lobbying.reduce(
    (sum, l) => sum + (l.amount ?? 0),
    0,
  );
  const maxAwardAmount = Math.max(
    ...awards.map((award) => award.obligation_amount ?? 0),
    0,
  );
  const topAwards = [...awards]
    .sort((a, b) => (b.obligation_amount ?? 0) - (a.obligation_amount ?? 0))
    .slice(0, 5);
  const relationshipGroups = edges.reduce(
    (groups, edge) => {
      const label = edgeTypeLabel(edge.edge_type);
      const current = groups.get(label) ?? {
        label,
        count: 0,
        amount: 0,
        confidenceTotal: 0,
        confidenceCount: 0,
      };
      current.count += 1;
      current.amount += edge.amount ?? 0;
      if (edge.confidence != null) {
        current.confidenceTotal += edge.confidence;
        current.confidenceCount += 1;
      }
      groups.set(label, current);
      return groups;
    },
    new Map<
      string,
      {
        label: string;
        count: number;
        amount: number;
        confidenceTotal: number;
        confidenceCount: number;
      }
    >(),
  );
  const relationshipSummary = Array.from(relationshipGroups.values()).sort(
    (a, b) => b.count - a.count,
  );

  return (
    <div className="min-h-screen text-white font-sans flex flex-col bg-black">
      <header className="w-full flex items-center justify-between px-8 py-6 border-b border-white/10">
        <Link href="/" className="ff-sans-wide text-3xl tracking-tighter text-white hover:opacity-70 transition-opacity">
          FundingFlow
        </Link>
        <Link href="/" className="ff-focus-ring font-mono text-[10px] uppercase tracking-widest text-slate-400 hover:text-[#DFFF00] transition-colors">
          Back to Search
        </Link>
      </header>

      <main className="flex-1 w-full mx-auto pb-16">
        <div className="grid grid-cols-1 md:grid-cols-12 border-b border-white/10">
          {/* Header Bento Box */}
          <div className="relative col-span-1 md:col-span-8 p-8 md:p-12 text-white flex flex-col justify-between min-h-[360px] overflow-hidden border-r border-white/10">
            <CropMarks className="text-white/10" />
            <div className="ff-serif absolute -right-6 bottom-0 hidden text-[20rem] leading-none text-white/[0.02] md:block select-none pointer-events-none">
              ID
            </div>
            <div className="relative z-10">
              <span className="font-mono text-[10px] font-black uppercase tracking-widest text-[#DFFF00] mb-8 inline-block">
                {e.entity_type}
              </span>
              <h1 className="ff-sans-wide text-5xl md:text-7xl font-black tracking-tighter mb-4 leading-[0.9]">
                {e.display_name}
              </h1>
            </div>
            <div className="font-mono text-[9px] uppercase tracking-widest font-semibold opacity-50 relative z-10">
              Created on {new Date(e.created_at).toLocaleDateString()}
            </div>
          </div>

          {/* Stats Bento Box */}
          <div className="relative overflow-hidden col-span-1 md:col-span-4 p-8 md:p-12 text-white flex flex-col justify-between min-h-[360px]">
            <CropMarks className="text-white/10" />
            <div className="mb-8">
              <h3 className="font-mono text-[10px] uppercase tracking-widest text-slate-500 mb-4 leading-none">
                Total Obligations
              </h3>
              <div className="ff-pixel text-[4rem] md:text-[5rem] tracking-tight font-black leading-none text-[#DFFF00]">
                <Decimals>{formatMoney(totalObligations)}</Decimals>
              </div>
            </div>
            <div className="grid grid-cols-3 gap-4 mt-auto pt-8 border-t border-white/10">
              <div>
                <h4 className="font-mono text-[9px] uppercase tracking-widest text-slate-500 leading-none mb-3">Awards</h4>
                <p className="ff-pixel text-3xl font-black leading-none text-white">{awards.length}</p>
              </div>
              <div>
                <h4 className="font-mono text-[9px] uppercase tracking-widest text-slate-500 leading-none mb-3">Edges</h4>
                <p className="ff-pixel text-3xl font-black leading-none text-white">{edges.length}</p>
              </div>
              <div>
                <h4 className="font-mono text-[9px] uppercase tracking-widest text-slate-500 leading-none mb-3">Lobbying</h4>
                <p className="ff-pixel text-3xl font-black leading-none text-white">{lobbying.length}</p>
              </div>
            </div>
          </div>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-2 border-b border-white/10">
          {/* Identifiers Bento Box */}
          {entity.identifiers.length > 0 ? (
            <div className="relative overflow-hidden p-8 md:p-12 border-r border-white/10">
              <CropMarks className="text-white/10" />
              <h3 className="font-mono text-[10px] uppercase tracking-widest text-slate-500 mb-8">Identifiers</h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-[1px] bg-white/10 border border-white/10">
                {entity.identifiers.map((id) => (
                  <div key={id.id} className="relative overflow-hidden bg-black p-6">
                    <CropMarks className="opacity-10" />
                    <div className="ff-pixel text-2xl font-black text-white leading-tight break-all">{id.identifier_value}</div>
                    <div className="font-mono text-[9px] uppercase tracking-widest text-slate-500 mt-4 leading-none">
                      {id.source} / {id.identifier_type}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ) : <div className="border-r border-white/10" />}

          {/* Aliases */}
          {entity.aliases.length > 0 ? (
            <div className="relative overflow-hidden p-8 md:p-12">
              <CropMarks className="text-white/10" />
              <h3 className="font-mono text-[10px] uppercase tracking-widest text-slate-500 mb-8">Aliases</h3>
              <div className="flex flex-wrap gap-[1px] bg-white/10 border border-white/10 p-[1px]">
                {entity.aliases.map((alias) => (
                  <span key={alias.id} className="bg-black px-5 py-3 text-[10px] font-bold uppercase tracking-widest font-mono text-slate-300">
                    {alias.alias}
                  </span>
                ))}
              </div>
            </div>
          ) : <div />}
        </div>

        {/* Tabs Control */}
        <div className="w-full border-b border-white/10 flex overflow-x-auto no-scrollbar">
          {(
            [
              ["awards", "Awards"],
              ["lobbying", "Lobbying"],
              ["finance", "Campaign Finance"],
              ["edges", "Relationships"],
              ["economic", "Economic Context"],
            ] as [Tab, string][]
          ).map(([key, label]) => (
            <button
              key={key}
              type="button"
              onClick={() => setTab(key)}
              className={`ff-focus-ring flex-1 font-mono text-[10px] uppercase tracking-widest px-8 py-6 transition-colors border-r border-white/10 last:border-r-0 whitespace-nowrap ${
                tab === key
                  ? "bg-[#DFFF00] text-black font-black"
                  : "bg-black text-slate-400 hover:bg-white/5 hover:text-white"
              }`}
            >
              {label}
            </button>
          ))}
        </div>

        {/* Tab Content */}
        <div className="w-full">
          {tab === "awards" && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-[1px] bg-white/10 border-b border-white/10">
              {awards.length === 0 ? (
                <div className="col-span-1 md:col-span-3 relative overflow-hidden bg-black p-12 text-center">
                  <CropMarks className="text-white/20" />
                  <p className="font-mono text-[10px] font-black uppercase tracking-widest text-slate-500">No awards found.</p>
                </div>
              ) : (
                <>
                  <div className="relative overflow-hidden col-span-1 bg-black p-8 md:p-12 text-white md:col-span-2 lg:col-span-3">
                    <CropMarks className="text-white/10" />
                    <div className="flex flex-col gap-2 md:flex-row md:items-end md:justify-between mb-12">
                      <div>
                        <p className="font-mono text-[10px] font-black uppercase tracking-widest text-[#DFFF00]">
                          Award concentration
                        </p>
                        <h3 className="ff-sans-wide mt-4 text-3xl font-black tracking-tighter text-white leading-none">
                          Largest obligations
                        </h3>
                      </div>
                      <p className="font-mono text-[10px] font-bold uppercase tracking-widest text-slate-400">
                        {formatMoney(totalObligations)} total across {awards.length} awards
                      </p>
                    </div>
                    <div className="grid grid-cols-1 gap-[1px] bg-white/10 border border-white/10">
                      {topAwards.map((award, index) => {
                        const width =
                          maxAwardAmount > 0
                            ? Math.max(((award.obligation_amount ?? 0) / maxAwardAmount) * 100, 4)
                            : 4;
                        return (
                          <div key={award.id} className="relative overflow-hidden bg-black p-6 hover:bg-white/5 transition-colors">
                            <CropMarks className="opacity-15 text-slate-500" />
                            <div className="flex items-start justify-between gap-6">
                              <div>
                                <p className="font-mono text-[9px] font-black uppercase tracking-widest text-slate-500 mb-2">
                                  Rank {index + 1}
                                </p>
                                <p className="text-base font-bold text-slate-200 tracking-tight line-clamp-2 max-w-2xl">
                                  {award.description ?? award.generated_unique_award_id}
                                </p>
                              </div>
                              <p className="ff-pixel shrink-0 text-3xl font-black text-[#DFFF00] leading-none">
                                <Decimals>{formatMoney(award.obligation_amount)}</Decimals>
                              </p>
                            </div>
                            <div className="mt-6 h-[2px] w-full bg-white/10">
                              <div className="h-full bg-[#DFFF00]" style={{ width: `${width}%` }} />
                            </div>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                  {awards.map((award) => {
                    return (
                      <div
                        key={award.id}
                        className="col-span-1 bg-black p-8 flex flex-col justify-between min-h-[280px] relative overflow-hidden transition-colors hover:bg-white/5"
                      >
                        <CropMarks className="opacity-15" />
                        <div>
                          <div className="flex justify-between items-start mb-6">
                            <span className="font-mono text-[9px] font-black uppercase tracking-widest text-slate-400 border border-white/10 px-3 py-1.5 rounded-full">
                              {award.award_type ?? "Award"}
                            </span>
                          </div>
                          <h4 className="ff-pixel text-4xl font-black mb-4 leading-none text-white tracking-tighter">
                            <Decimals>{formatMoney(award.obligation_amount)}</Decimals>
                          </h4>
                          {award.description && (
                            <p className="text-xs font-medium mt-4 line-clamp-3 opacity-60 leading-relaxed">
                              {award.description}
                            </p>
                          )}
                        </div>
                        <div className="mt-8 pt-6 border-t border-white/10 flex flex-col gap-2 font-mono text-[9px] font-semibold uppercase tracking-widest text-slate-500">
                          {award.period_start && (
                            <span>Start: {award.period_start}</span>
                          )}
                          {award.place_of_performance && (
                            <span>Loc: {award.place_of_performance}</span>
                          )}
                          <span className="truncate">ID: {award.generated_unique_award_id}</span>
                        </div>
                      </div>
                    );
                  })}
                </>
              )}
            </div>
          )}

          {tab === "lobbying" && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-[1px] bg-white/10 border-b border-white/10">
              {lobbying.length === 0 ? (
                <div className="col-span-1 md:col-span-3 relative overflow-hidden bg-black p-12 text-center">
                  <CropMarks className="text-slate-800" />
                  <p className="font-mono text-[10px] font-black uppercase tracking-widest text-slate-500">No lobbying filings found.</p>
                </div>
              ) : (
                <>
                  <div className="col-span-1 md:col-span-3 relative overflow-hidden bg-[#DFFF00] p-8 text-black shadow-[inset_0px_0px_0px_1px_rgba(0,0,0,0.1)]">
                    <CropMarks className="opacity-20" />
                    <h3 className="ff-sans-wide text-3xl font-black text-black leading-none mb-6">Lobbying Summary</h3>
                    <div className="flex gap-12 mt-4">
                      <div>
                        <p className="font-mono text-[10px] font-black uppercase tracking-widest opacity-60 leading-none mb-3">Total Filings</p>
                        <p className="ff-pixel text-4xl font-black leading-none">{lobbying.length}</p>
                      </div>
                      <div>
                        <p className="font-mono text-[10px] font-black uppercase tracking-widest opacity-60 leading-none mb-3">Total Amount</p>
                        <p className="ff-pixel text-4xl font-black leading-none"><Decimals>{formatMoney(totalLobbyingAmount)}</Decimals></p>
                      </div>
                    </div>
                  </div>
                  {lobbying.map((filing) => {
                    return (
                      <div
                        key={filing.id}
                        className="col-span-1 bg-black p-8 flex flex-col justify-between min-h-[240px] relative overflow-hidden transition-colors hover:bg-white/5"
                      >
                        <CropMarks className="opacity-15" />
                        <div>
                          <span className="font-mono text-[9px] font-black uppercase tracking-widest text-slate-400 border border-white/10 px-3 py-1.5 rounded-full inline-block mb-6">
                            {filing.filing_type ?? "Lobbying Filing"}
                          </span>
                          <h4 className="ff-pixel text-4xl font-black tracking-tighter mb-2 leading-none text-white">
                            <Decimals>{formatMoney(filing.amount)}</Decimals>
                          </h4>
                        </div>
                        <div className="mt-8 pt-6 border-t border-white/10 flex flex-col gap-2 font-mono text-[9px] font-semibold uppercase tracking-widest text-slate-500">
                          {filing.filing_year && <span>Year: {filing.filing_year}</span>}
                          {filing.income_or_expense && <span>{filing.income_or_expense}</span>}
                          {filing.filing_period && <span>Period: {filing.filing_period}</span>}
                          {filing.filing_uuid && <span className="truncate">ID: {filing.filing_uuid}</span>}
                        </div>
                      </div>
                    );
                  })}
                </>
              )}
            </div>
          )}
          {tab === "finance" && (
            <div className="border-b border-white/10 p-8 md:p-12">
              {contributions?.summary ? (
                (() => {
                  const s = contributions.summary!;
                  const maxRecipient = Math.max(...s.by_recipient.map(r => r.total_amount), 0.01);
                  return (
                    <>
                      <div className="mb-12">
                        <h3 className="font-mono text-[10px] uppercase tracking-widest text-slate-500 mb-4">Total Contributions</h3>
                        <div className="ff-pixel text-[5rem] tracking-tight font-black leading-none text-[#DFFF00] mb-4">
                          <Decimals>{formatMoney(s.total_amount)}</Decimals>
                        </div>
                        <p className="font-mono text-[10px] uppercase tracking-widest text-slate-500">
                          {s.transaction_count} transactions
                        </p>
                      </div>

                      {contributions.candidate_info && (
                        <div className="mb-12 border border-white/10 p-6 rounded-[1rem]">
                          <h4 className="font-mono text-[10px] uppercase tracking-widest text-[#DFFF00] mb-4">Candidate Info</h4>
                          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                            <div><span className="text-slate-500 text-xs uppercase">FEC ID</span><p className="font-mono">{contributions.candidate_info.candidate_fec_id}</p></div>
                            <div><span className="text-slate-500 text-xs uppercase">Party</span><p>{contributions.candidate_info.party}</p></div>
                            <div><span className="text-slate-500 text-xs uppercase">Office</span><p>{contributions.candidate_info.office}</p></div>
                            <div><span className="text-slate-500 text-xs uppercase">State</span><p>{contributions.candidate_info.office_state}</p></div>
                          </div>
                        </div>
                      )}

                      {contributions.committee_info && (
                        <div className="mb-12 border border-white/10 p-6 rounded-[1rem]">
                          <h4 className="font-mono text-[10px] uppercase tracking-widest text-[#DFFF00] mb-4">Committee Info</h4>
                          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                            <div><span className="text-slate-500 text-xs uppercase">FEC ID</span><p className="font-mono">{contributions.committee_info.committee_fec_id}</p></div>
                            <div><span className="text-slate-500 text-xs uppercase">Type</span><p>{contributions.committee_info.committee_type}</p></div>
                            <div><span className="text-slate-500 text-xs uppercase">Designation</span><p>{contributions.committee_info.committee_designation}</p></div>
                            <div><span className="text-slate-500 text-xs uppercase">Party</span><p>{contributions.committee_info.party}</p></div>
                          </div>
                        </div>
                      )}

                      {s.by_recipient.length > 0 && (
                        <div className="mb-12">
                          <h4 className="font-mono text-[10px] uppercase tracking-widest text-slate-500 mb-6">Top Recipients</h4>
                          <div className="space-y-2">
                            {s.by_recipient.map((r, i) => (
                              <div key={i} className="flex items-center gap-4">
                                <div className="font-mono text-[10px] text-slate-500 w-6">{i + 1}</div>
                                {r.recipient_entity_id ? (
                                  <Link href={`/entities/${r.recipient_entity_id}`} className="flex-1 text-sm hover:text-[#DFFF00] transition-colors">
                                    {r.recipient_name}
                                  </Link>
                                ) : (
                                  <div className="flex-1 text-sm text-slate-400">{r.recipient_name}</div>
                                )}
                                <div className="w-32 md:w-48 bg-white/10 h-2 rounded-sm overflow-hidden">
                                  <div className="bg-[#DFFF00] h-full" style={{ width: `${(r.total_amount / maxRecipient) * 100}%` }} />
                                </div>
                                <div className="ff-pixel text-sm font-black w-24 text-right">{formatMoney(r.total_amount)}</div>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}

                      {s.by_cycle.length > 0 && (
                        <div className="mb-12">
                          <h4 className="font-mono text-[10px] uppercase tracking-widest text-slate-500 mb-6">By Cycle</h4>
                          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                            {s.by_cycle.map((c, i) => (
                              <div key={i} className="border border-white/10 p-4 rounded-[0.5rem]">
                                <div className="font-mono text-[10px] text-slate-500">{c.cycle}</div>
                                <div className="ff-pixel text-2xl font-black">{formatMoney(c.total_amount)}</div>
                                <div className="font-mono text-[9px] text-slate-500">{c.transaction_count} txns</div>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}

                      {contributions.transactions.length > 0 && (
                        <div>
                          <h4 className="font-mono text-[10px] uppercase tracking-widest text-slate-500 mb-6">Recent Transactions</h4>
                          <div className="overflow-x-auto">
                            <table className="w-full text-sm">
                              <thead>
                                <tr className="font-mono text-[9px] uppercase tracking-widest text-slate-500 border-b border-white/10">
                                  <th className="text-left pb-2 pr-4">Date</th>
                                  <th className="text-left pb-2 pr-4">Amount</th>
                                  <th className="text-left pb-2 pr-4">Recipient</th>
                                  <th className="text-left pb-2 pr-4">Type</th>
                                  <th className="text-left pb-2 pr-4">Memo</th>
                                </tr>
                              </thead>
                              <tbody>
                                {contributions.transactions.slice(0, 50).map((t, i) => (
                                  <tr key={i} className="border-b border-white/5 hover:bg-white/5">
                                    <td className="py-2 pr-4 font-mono text-xs">{t.date ?? "N/A"}</td>
                                    <td className="py-2 pr-4 ff-pixel font-black text-[#DFFF00]">{formatMoney(t.amount ?? 0)}</td>
                                    <td className="py-2 pr-4 text-xs">{t.recipient_committee_name ?? t.committee_fec_id ?? "N/A"}</td>
                                    <td className="py-2 pr-4 font-mono text-xs text-slate-400">{t.transaction_type ?? "N/A"}</td>
                                    <td className="py-2 pr-4 text-xs text-slate-400 max-w-xs truncate">{t.memo_text ?? ""}</td>
                                  </tr>
                                ))}
                              </tbody>
                            </table>
                          </div>
                        </div>
                      )}
                    </>
                  );
                })()
              ) : (
                <div className="ff-terminal relative overflow-hidden rounded-[1.75rem] border-2 border-black p-12 text-center text-xl font-medium text-gray-400">
                  <CropMarks className="text-slate-800" />
                  No campaign finance data recorded.
                </div>
              )}
            </div>
          )}
          {tab === "edges" && (
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-[1px] bg-white/10 border-b border-white/10">
              {edges.length === 0 ? (
                <div className="col-span-1 md:col-span-3 ff-terminal relative overflow-hidden rounded-[1.75rem] border-2 border-black p-12 text-center text-xl font-medium text-gray-400">
                  <CropMarks className="text-slate-800" />
                  No relationships recorded.
                </div>
              ) : (
                <>
                  <div className="ff-blueprint relative overflow-hidden col-span-1 border-2 border-black rounded-[1.75rem] p-6 md:col-span-2 lg:col-span-3 shadow-[6px_6px_0px_0px_rgba(0,0,0,0.9)]">
                    <CropMarks className="text-white/20" />
                    <div className="flex flex-col gap-5 lg:grid lg:grid-cols-[1.1fr_1.4fr]">
                      <div>
                        <p className="ff-micro text-xs font-black text-[#f5f2e8]/70">
                          Relationship graph
                        </p>
                        <h3 className="ff-sans-wide mt-2 text-3xl font-black tracking-tight text-white leading-none">
                          {e.display_name}
                        </h3>
                        <p className="mt-3 text-sm font-semibold text-[#f5f2e8]/70 leading-normal">
                          {edges.length} recorded relationships grouped by edge type, source amount, and confidence.
                        </p>
                        <div className="ff-paper relative overflow-hidden mt-6 rounded-[1.5rem] p-5 text-black">
                          <CropMarks className="opacity-20" />
                          <p className="ff-micro text-xs font-black opacity-55">
                            Center node
                          </p>
                          <p className="ff-pixel mt-2 text-4xl font-black leading-none">
                            {edges.length}
                          </p>
                          <p className="mt-1 text-xs font-bold opacity-55">
                            Connected edge records
                          </p>
                        </div>
                      </div>
                      <div className="space-y-3.5">
                        {relationshipSummary.map((group, index) => {
                          const confidence =
                            group.confidenceCount > 0
                              ? (group.confidenceTotal / group.confidenceCount) * 100
                              : null;
                          return (
                            <div key={group.label} className="relative overflow-hidden rounded-2xl bg-black/25 p-4 border border-white/5 shadow-[2px_2px_0px_0px_rgba(0,0,0,0.15)]">
                              <CropMarks className="opacity-15 text-slate-500" />
                              <div className="flex items-start justify-between gap-4">
                                <div>
                                  <p className="ff-micro text-[9px] font-black text-[#f5f2e8]/60 leading-none">
                                    Type {index + 1}
                                  </p>
                                  <p className="mt-1 font-black text-slate-200 leading-tight">{group.label}</p>
                                  <p className="mt-1 font-mono text-[10px] font-semibold text-[#f5f2e8]/60 leading-none">
                                    {group.count} edges
                                    {confidence != null ? ` / ${confidence.toFixed(0)}% conf` : ""}
                                  </p>
                                </div>
                                <p className="ff-pixel shrink-0 text-2xl font-black text-[#f8df1d] leading-none">
                                  <Decimals>{group.amount > 0 ? formatMoney(group.amount) : String(group.count)}</Decimals>
                                </p>
                              </div>
                              <div className="mt-4 h-2.5 overflow-hidden border border-black bg-white/10">
                                <div
                                  className={index % 2 === 0 ? "h-full bg-[#f8df1d]" : "h-full bg-[#a69ad1]"}
                                  style={{ width: `${Math.max((group.count / edges.length) * 100, 8)}%` }}
                                />
                              </div>
                            </div>
                          );
                        })}
                      </div>
                    </div>
                  </div>
                  {edges.map((edge, i) => {
                    const bgs = [
                      "ff-terminal text-white shadow-[4px_4px_0px_0px_rgba(255,255,255,0.04)]",
                      "bg-[#e96b4c] text-black shadow-[4px_4px_0px_0px_rgba(0,0,0,0.85)]",
                      "bg-[#f0ede6] text-black shadow-[4px_4px_0px_0px_rgba(0,0,0,0.85)]",
                    ];
                    const colorClass = bgs[i % bgs.length];
                    return (
                      <div
                        key={edge.id}
                        className={`col-span-1 rounded-[1.75rem] border-2 border-black p-8 flex flex-col justify-between min-h-[250px] relative overflow-hidden transition-all hover:shadow-none ${colorClass}`}
                      >
                        <CropMarks className="opacity-20" />
                        <div>
                          <div className="flex justify-between items-start mb-4">
                            <span
                              className={`px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-wider border border-current/15 ${
                                colorClass.includes("text-white")
                                  ? "bg-white/10"
                                  : "bg-black/5"
                              }`}
                            >
                              {edgeTypeLabel(edge.edge_type)}
                            </span>
                          </div>
                          {edge.amount != null && (
                            <h4 className="ff-pixel text-4xl font-bold tracking-tight mb-2 leading-none">
                              <Decimals>{formatMoney(edge.amount)}</Decimals>
                            </h4>
                          )}
                          {edge.description && (
                            <p
                              className={`text-sm font-medium mt-4 line-clamp-3 ${
                                colorClass.includes("text-white")
                                  ? "opacity-80"
                                  : "opacity-70"
                              }`}
                            >
                              {edge.description}
                            </p>
                          )}
                        </div>
                        <div className="mt-8 pt-4 border-t border-current/15 flex flex-col gap-1.5 font-mono text-[10px] font-bold uppercase tracking-wider opacity-75">
                          {edge.started_on && (
                            <span>Start: {edge.started_on}</span>
                          )}
                          <span>Source: {edge.source}</span>
                          {edge.confidence != null && (
                            <span>
                              Conf: {(edge.confidence * 100).toFixed(0)}%
                            </span>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </>
              )}
            </div>
          )}

          {tab === "economic" && (
            <div className="space-y-8">
              {economicContext.length === 0 ? (
                <div className="ff-terminal relative overflow-hidden rounded-[1.75rem] border-2 border-black p-12 text-center text-xl font-medium text-gray-400">
                  <CropMarks className="text-slate-800" />
                  No geographic economic data linked to this entity.
                </div>
              ) : (
                economicContext.map((profile) => {
                  const metricsByCategory = new Map<string, typeof profile.latest_metrics>();
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
                  sections.forEach((section) => {
                    metricsByCategory.set(
                      section.key,
                      profile.latest_metrics.filter(
                        (item) => item.metric.category === section.key,
                      ),
                    );
                  });

                  return (
                    <div key={profile.geo.geo_id} className="space-y-4">
                      <div className="relative overflow-hidden border-2 border-black bg-[#d2f1ec] rounded-[1.75rem] p-8 text-black mb-4 shadow-[6px_6px_0px_0px_rgba(0,0,0,1)]">
                        <CropMarks className="opacity-25" />
                        <div className="flex items-center justify-between">
                          <div>
                            <span className="bg-black/10 border border-black/10 px-3 py-1 rounded-full text-[10px] font-black uppercase tracking-widest leading-none">
                              {profile.geo.geo_type}
                            </span>
                            <h2 className="ff-sans-wide text-3xl font-black mt-2.5 leading-none">{profile.geo.name}</h2>
                          </div>
                          <Link
                            href={`/states/${profile.geo.geo_id}`}
                            className="ff-focus-ring font-mono text-xs uppercase tracking-wider bg-black text-white px-4 py-2 border-2 border-black hover:bg-black/85 transition shadow-[3px_3px_0px_0px_rgba(255,255,255,0.2)] rounded-none"
                          >
                            Full Profile
                          </Link>
                        </div>
                      </div>

                      {/* Derived Metrics */}
                      {profile.derived_metrics.length > 0 && (
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-3.5 mb-4">
                          {profile.derived_metrics.map((metric) => (
                            <div key={metric.metric_id} className="relative overflow-hidden rounded-xl border-2 border-black bg-[#9cb199] p-4 text-black shadow-[4px_4px_0px_0px_rgba(0,0,0,1)]">
                              <CropMarks className="opacity-20" />
                              <p className="ff-micro text-[10px] font-black opacity-70 leading-tight">
                                {derivedMetricLabel(metric.metric_id)}
                              </p>
                              <p className="ff-pixel mt-2.5 text-2xl font-black leading-none">
                                <Decimals>
                                  {metric.metric_id.includes("intensity")
                                    ? `${(metric.value * 100).toFixed(3)}%`
                                    : metric.value.toFixed(1)}
                                </Decimals>
                              </p>
                              <p className="mt-2.5 font-mono text-[9px] font-semibold opacity-60 leading-none border-t border-black/10 pt-2">
                                {metric.formula}
                              </p>
                            </div>
                          ))}
                        </div>
                      )}

                      {/* Top Public-Money Recipients */}
                      {profile.public_money.top_recipients.length > 0 && (
                        <div className="relative overflow-hidden rounded-[1.25rem] border-2 border-black bg-white/5 shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] p-5 mb-4">
                          <CropMarks className="opacity-20" />
                          <h3 className="ff-sans-wide text-lg font-black text-white leading-none mb-4">
                            Top Public-Money Recipients in {profile.geo.name}
                          </h3>
                          <div className="divide-y divide-white/5">
                            {profile.public_money.top_recipients.map((recipient) => (
                              <div
                                key={`${recipient.entity_id}-${recipient.display_name}`}
                                className="flex items-center justify-between gap-4 py-3 font-semibold text-slate-300"
                              >
                                <div>
                                  <p className="text-sm text-slate-200">{recipient.display_name}</p>
                                  <p className="font-mono text-[10px] text-slate-500 mt-1">
                                    {recipient.award_count.toLocaleString()} award records
                                  </p>
                                </div>
                                <p className="ff-pixel shrink-0 text-2xl font-black text-white leading-none">
                                  <Decimals>{formatMoney(recipient.total_obligations)}</Decimals>
                                </p>
                              </div>
                            ))}
                          </div>
                        </div>
                      )}

                      {/* Wage Gap by Sex */}
                      {(() => {
                        const male = profile.latest_metrics.find(
                          (item) => item.observation.metric_id === "median_earnings_male",
                        );
                        const female = profile.latest_metrics.find(
                          (item) => item.observation.metric_id === "median_earnings_female",
                        );
                        if (!male || !female) return null;
                        const ratio =
                          male.observation.value > 0
                            ? (female.observation.value / male.observation.value) * 100
                            : 0;
                        return (
                          <div className="relative overflow-hidden rounded-[1.25rem] border-2 border-black bg-white/5 shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] p-5 mb-4">
                            <CropMarks className="opacity-20" />
                            <h3 className="ff-sans-wide text-lg font-black text-white leading-none mb-4">Wage Gap by Sex</h3>
                            <div className="grid grid-cols-3 gap-4 items-center">
                              <div>
                                <p className="ff-micro text-[10px] text-slate-500 mb-1 leading-none">Men</p>
                                <p className="ff-pixel text-2xl font-black leading-none text-white">
                                  <Decimals>{formatMoney(male.observation.value)}</Decimals>
                                </p>
                              </div>
                              <div>
                                <p className="ff-micro text-[10px] text-slate-500 mb-1 leading-none">Women</p>
                                <p className="ff-pixel text-2xl font-black leading-none text-white">
                                  <Decimals>{formatMoney(female.observation.value)}</Decimals>
                                </p>
                              </div>
                              <div className="relative overflow-hidden rounded-lg border-2 border-black bg-[#a69ad1] p-3 text-center text-black shadow-[3px_3px_0px_0px_rgba(0,0,0,1)]">
                                <CropMarks className="opacity-15" />
                                <p className="ff-micro text-[9px] font-black opacity-60 leading-none mb-1">
                                  Ratio
                                </p>
                                <p className="ff-pixel text-xl font-black leading-none text-black">
                                  <Decimals>{ratio.toFixed(1)}%</Decimals>
                                </p>
                              </div>
                            </div>
                          </div>
                        );
                      })()}

                      {/* Income Distribution */}
                      {(() => {
                        const quintiles = [
                          { key: "quintile_income_bottom", label: "Lowest 20%" },
                          { key: "quintile_income_second", label: "Second 20%" },
                          { key: "quintile_income_third", label: "Third 20%" },
                          { key: "quintile_income_fourth", label: "Fourth 20%" },
                          { key: "quintile_income_top", label: "Highest 20%" },
                        ];
                        const items = quintiles
                          .map((q) => {
                            const match = profile.latest_metrics.find(
                              (item) => item.observation.metric_id === q.key,
                            );
                            return match
                              ? { ...q, value: match.observation.value, date: match.observation.date }
                              : { ...q, value: null, date: null };
                          })
                          .filter((q) => q.value != null);
                        if (items.length === 0) return null;
                        const max = Math.max(...items.map((q) => q.value!));
                        return (
                          <div className="relative overflow-hidden rounded-[1.25rem] border-2 border-black bg-white/5 shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] p-5 mb-4">
                            <CropMarks className="opacity-20" />
                            <h3 className="ff-sans-wide text-lg font-black text-white leading-none mb-4">Income Distribution</h3>
                            <div className="space-y-3.5">
                              {items.map((q) => (
                                <div key={q.key} className="space-y-1">
                                  <div className="flex justify-between text-sm font-semibold">
                                    <span className="text-slate-300">{q.label}</span>
                                    <span className="ff-pixel text-lg font-bold text-white leading-none">
                                      <Decimals>{formatMoney(q.value!)}</Decimals>
                                    </span>
                                  </div>
                                  <div className="h-3 overflow-hidden border border-black bg-white/10">
                                    <div
                                      className="h-full bg-[#d2f1ec]"
                                      style={{
                                        width: `${Math.max((q.value! / max) * 100, 2)}%`,
                                      }}
                                    />
                                  </div>
                                </div>
                              ))}
                            </div>
                          </div>
                        );
                      })()}

                      {/* Metric History Sparklines */}
                      {(() => {
                        const charts = [
                          { key: "unemployment_rate", label: "Unemployment rate", unit: "percent" as const },
                          {
                            key: "avg_hourly_earnings",
                            label: "Avg hourly earnings",
                            unit: "usd_per_hour" as const,
                          },
                          { key: "fhfa_hpi_yoy", label: "HPI YoY change", unit: "percent" as const },
                        ];
                        const seriesData = charts.map((chart) => {
                          const series = profile.latest_metrics
                            .filter((item) => item.observation.metric_id === chart.key)
                            .sort((a, b) =>
                              a.observation.date.localeCompare(b.observation.date),
                            );
                          return { ...chart, series };
                        });
                        const hasData = seriesData.some((s) => s.series.length >= 2);
                        if (!hasData) return null;
                        return (
                          <div className="relative overflow-hidden rounded-[1.25rem] border-2 border-black bg-white/5 shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] p-5 mb-4">
                            <CropMarks className="opacity-20" />
                            <h3 className="ff-sans-wide text-lg font-black text-white leading-none mb-4">Metric History</h3>
                            <div className="grid gap-3.5 sm:grid-cols-2 lg:grid-cols-3">
                              {seriesData.map((chart) => {
                                if (chart.series.length < 2) return null;
                                const values = chart.series.map(
                                  (s) => s.observation.value,
                                );
                                const min = Math.min(...values);
                                const max = Math.max(...values);
                                const range = max - min || 1;
                                return (
                                  <div key={chart.key} className="relative overflow-hidden rounded-lg border border-black/20 bg-black/45 p-4 shadow-[2px_2px_0px_0px_rgba(0,0,0,0.3)]">
                                    <CropMarks className="text-slate-800 opacity-40" />
                                    <p className="ff-micro text-[10px] font-black text-slate-500 mb-2 leading-none">
                                      {chart.label}
                                    </p>
                                    <div className="flex items-end gap-[2px] h-16">
                                      {chart.series.slice(-36).map((s, i) => (
                                        <div
                                          key={i}
                                          className="flex-1 bg-[#d2f1ec]/60 hover:bg-[#d2f1ec] rounded-t-sm transition"
                                          style={{
                                            height: `${((s.observation.value - min) / range) * 100}%`,
                                          }}
                                          title={`${s.observation.date}: ${s.observation.value}`}
                                        />
                                      ))}
                                    </div>
                                    <div className="mt-2.5 flex justify-between font-mono text-[9px] text-slate-600 leading-none">
                                      <span>{chart.series[0].observation.date}</span>
                                      <span>
                                        {
                                          chart.series[chart.series.length - 1]
                                            .observation.date
                                        }
                                      </span>
                                    </div>
                                  </div>
                                );
                              })}
                            </div>
                          </div>
                        );
                      })()}

                      {/* All Metrics by Category */}
                      <div className="grid gap-4 md:grid-cols-2">
                        {sections.map((section) => {
                          const metrics = metricsByCategory.get(section.key) ?? [];
                          if (metrics.length === 0) return null;
                          return (
                            <div
                              key={section.key}
                              className="relative overflow-hidden rounded-[1.25rem] border-2 border-black bg-white/5 shadow-[6px_6px_0px_0px_rgba(0,0,0,1)] p-5"
                            >
                              <CropMarks className="opacity-20" />
                              <h4 className="ff-sans-wide text-base font-black text-white leading-none mb-3">
                                {section.title}
                              </h4>
                              <div className="divide-y divide-white/10">
                                {metrics.map((item) => (
                                  <div
                                    key={item.metric.metric_id}
                                    className="py-3"
                                  >
                                    <div className="flex items-start justify-between gap-4">
                                      <div>
                                        <p className="font-semibold text-sm text-slate-200">
                                          {item.metric.name}
                                        </p>
                                        <p className="mt-1 font-mono text-[10px] text-slate-500 leading-none">
                                          {item.source.agency} /{" "}
                                          {item.source.dataset}
                                        </p>
                                      </div>
                                      <p className="ff-pixel shrink-0 text-xl font-black text-[#d2f1ec] leading-none">
                                        <Decimals>
                                          {formatMetricValue(
                                            item.observation.value,
                                            item.metric.unit,
                                          )}
                                        </Decimals>
                                      </p>
                                    </div>
                                    <p className="mt-2.5 font-mono text-[9px] text-slate-600 leading-none">
                                      Date: {item.observation.date}. Vintage:{" "}
                                      {item.observation.vintage_date}.
                                    </p>
                                  </div>
                                ))}
                              </div>
                            </div>
                          );
                        })}
                      </div>
                    </div>
                  );
                })
              )}
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
