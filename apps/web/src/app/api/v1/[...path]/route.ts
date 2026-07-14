import { NextRequest, NextResponse } from "next/server";

export const dynamic = "force-dynamic";
export const runtime = "nodejs";

const REQUEST_TIMEOUT_MS = 15_000;
const HOP_BY_HOP_HEADERS = [
  "connection",
  "keep-alive",
  "proxy-authenticate",
  "proxy-authorization",
  "te",
  "trailer",
  "transfer-encoding",
  "upgrade",
];

type RouteContext = {
  params: Promise<{ path: string[] }>;
};

export async function GET(request: NextRequest, context: RouteContext) {
  const { path } = await context.params;
  const apiOrigin = process.env.API_URL ?? "http://localhost:3001";

  try {
    const target = new URL(
      `/api/v1/${path.map((segment) => encodeURIComponent(segment)).join("/")}`,
      apiOrigin,
    );
    request.nextUrl.searchParams.forEach((value, key) => {
      target.searchParams.append(key, value);
    });

    const upstream = await fetch(target, {
      method: "GET",
      headers: { Accept: request.headers.get("accept") ?? "application/json" },
      cache: "no-store",
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
    const headers = new Headers(upstream.headers);
    HOP_BY_HOP_HEADERS.forEach((name) => headers.delete(name));

    return new NextResponse(upstream.body, {
      status: upstream.status,
      statusText: upstream.statusText,
      headers,
    });
  } catch (error) {
    console.error("FundingFlow API proxy failed", error);
    return NextResponse.json(
      {
        error: "api_unavailable",
        message: "FundingFlow could not reach the API service.",
      },
      { status: 502 },
    );
  }
}
