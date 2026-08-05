import { useQueries } from "@tanstack/react-query";
import { useStore } from "@tanstack/react-store";
import { Link } from "@tanstack/react-router";
import { ChevronRight } from "lucide-react";
import { api, scanKeys } from "@/lib/api";
import { recentScansStore } from "@/lib/store";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { StatusBadge } from "@/components/status-badge";

export function RecentScans() {
  const scans = useStore(recentScansStore, (state) => state);
  const statuses = useQueries({
    queries: scans.map((scan) => ({
      queryKey: scanKeys.detail(scan.scan_id),
      queryFn: () => api.getScan(scan.scan_id),
      refetchInterval: (query: { state: { data?: { status?: string } } }) =>
        query.state.data?.status === "succeeded" || query.state.data?.status === "failed" ? false : 2000,
    })),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle>Recent scans</CardTitle>
        <CardDescription>Kept in this browser. {scans.length} tracked.</CardDescription>
      </CardHeader>
      <CardContent className="flex flex-col gap-2">
        {scans.length === 0 ? (
          <p className="text-sm text-muted-foreground">No scans yet — queue one above.</p>
        ) : (
          scans.map((scan, index) => {
            const status = statuses[index]?.data?.status ?? scan.kind;
            return (
              <Link
                key={scan.scan_id}
                to="/scans/$scanId"
                params={{ scanId: scan.scan_id }}
                className="flex items-center justify-between rounded-lg border border-border px-3 py-2 transition-colors hover:bg-muted/40"
              >
                <div className="flex min-w-0 flex-col">
                  <span className="truncate text-sm font-medium">{scan.target}</span>
                  <span className="font-mono text-xs text-muted-foreground">{scan.scan_id}</span>
                </div>
                <div className="flex shrink-0 items-center gap-2">
                  <Badge variant="outline">{scan.engine}</Badge>
                  {statuses[index]?.data ? <StatusBadge status={status} /> : null}
                  <ChevronRight className="size-4 text-muted-foreground" />
                </div>
              </Link>
            );
          })
        )}
      </CardContent>
    </Card>
  );
}
