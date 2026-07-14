import type { UnifiedSearchResult } from "./types";

const REQUEST_TIMEOUT_MS = 15_000;

export async function unifiedSearchClient(
  query: string,
  limit = 20,
  offset = 0,
): Promise<UnifiedSearchResult> {
  const normalizedQuery = query.trim();
  if (!normalizedQuery) {
    throw new Error("Search query is required");
  }

  const params = new URLSearchParams({
    q: normalizedQuery,
    limit: String(limit),
    offset: String(offset),
  });
  const controller = new AbortController();
  const timeout = window.setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

  try {
    const response = await fetch(`/api/v1/search?${params}`, {
      headers: { Accept: "application/json" },
      signal: controller.signal,
    });
    if (!response.ok) {
      throw new Error(`Unified search failed with status ${response.status}`);
    }
    return (await response.json()) as UnifiedSearchResult;
  } finally {
    window.clearTimeout(timeout);
  }
}
