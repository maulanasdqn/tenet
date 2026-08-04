import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { api, scanKeys } from "@/lib/api";
import type { Scan } from "@/lib/types";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { StatusBadge } from "@/components/status-badge";
import { FindingsTable } from "@/components/findings-table";
import { EndpointsTable } from "@/components/endpoints-table";
import { OpenapiView } from "@/components/openapi-view";

type Tab = "findings" | "endpoints" | "openapi";

const TABS: { id: Tab; label: string }[] = [
  { id: "findings", label: "Findings" },
  { id: "endpoints", label: "Endpoints" },
  { id: "openapi", label: "OpenAPI" },
];

function isDone(scan?: Scan) {
  return scan?.status === "succeeded" || scan?.status === "failed";
}

export function ScanDetail({ scanId }: { scanId: string }) {
  const [tab, setTab] = useState<Tab>("findings");
  const query = useQuery({
    queryKey: scanKeys.detail(scanId),
    queryFn: () => api.getScan(scanId),
    refetchInterval: (q) => (isDone(q.state.data) ? false : 1500),
  });

  const scan = query.data;

  return (
    <div className="flex flex-col gap-6">
      <div>
        <Link
          to="/"
          className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
        >
          <ArrowLeft className="size-4" /> Back
        </Link>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex flex-wrap items-center gap-3">
            {scan ? <StatusBadge status={scan.status} /> : <Skeleton className="h-5 w-16" />}
            <span className="truncate">{scan?.target ?? scanId}</span>
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-2 text-sm">
          {query.isLoading ? (
            <Skeleton className="h-12 w-full" />
          ) : query.isError ? (
            <p className="text-destructive">{(query.error as Error).message}</p>
          ) : scan ? (
            <div className="flex flex-wrap items-center gap-2 text-muted-foreground">
              <Badge variant="outline">{scan.kind}</Badge>
              <Badge variant="outline">{scan.engine}</Badge>
              <span className="font-mono text-xs">{scan.scan_id}</span>
              <span>· attempt {scan.attempts}/{scan.max_attempts}</span>
              {scan.error ? <span className="text-destructive">· {scan.error}</span> : null}
              {!isDone(scan) ? <span className="animate-pulse">· working…</span> : null}
            </div>
          ) : null}
        </CardContent>
      </Card>

      <div className="flex gap-1 border-b border-border">
        {TABS.map((entry) => (
          <button
            key={entry.id}
            onClick={() => setTab(entry.id)}
            className={`-mb-px border-b-2 px-4 py-2 text-sm font-medium transition-colors ${
              tab === entry.id
                ? "border-primary text-foreground"
                : "border-transparent text-muted-foreground hover:text-foreground"
            }`}
          >
            {entry.label}
          </button>
        ))}
      </div>

      {tab === "findings" ? <FindingsTable scanId={scanId} /> : null}
      {tab === "endpoints" ? <EndpointsTable scanId={scanId} /> : null}
      {tab === "openapi" ? <OpenapiView scanId={scanId} /> : null}
    </div>
  );
}
