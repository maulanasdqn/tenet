import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ColumnDef } from "@tanstack/react-table";
import { api, scanKeys } from "@/lib/api";
import type { EndpointRow } from "@/lib/types";
import { DataTable } from "@/components/data-table";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Confidence } from "@/components/status-badge";

const METHOD_TONE: Record<string, string> = {
  get: "text-sky-400",
  post: "text-emerald-400",
  put: "text-amber-400",
  patch: "text-amber-400",
  delete: "text-red-400",
};

export function EndpointsTable({ scanId }: { scanId: string }) {
  const query = useQuery({
    queryKey: scanKeys.endpoints(scanId),
    queryFn: () => api.getEndpoints(scanId),
  });

  const columns = useMemo<ColumnDef<EndpointRow, unknown>[]>(
    () => [
      {
        accessorKey: "method",
        header: "Method",
        cell: ({ row }) => (
          <span className={`font-mono text-xs font-semibold uppercase ${METHOD_TONE[row.original.method] ?? ""}`}>
            {row.original.method}
          </span>
        ),
      },
      {
        accessorKey: "path",
        header: "Path",
        cell: ({ row }) => <span className="font-mono text-xs">{row.original.path}</span>,
      },
      {
        accessorKey: "base_url",
        header: "Origin",
        cell: ({ row }) => (
          <span className="font-mono text-xs text-muted-foreground">{row.original.base_url ?? "—"}</span>
        ),
      },
      {
        accessorKey: "source",
        header: "Source",
        cell: ({ row }) => (
          <Badge variant={row.original.source === "runtime" ? "success" : "secondary"}>
            {row.original.source === "runtime" ? "runtime" : "static"}
          </Badge>
        ),
      },
      {
        accessorKey: "confidence",
        header: "Confidence",
        cell: ({ row }) => <Confidence value={row.original.confidence} />,
      },
    ],
    [],
  );

  if (query.isLoading) return <Skeleton className="h-64 w-full" />;
  if (query.isError) return <p className="text-sm text-destructive">{(query.error as Error).message}</p>;

  return <DataTable columns={columns} data={query.data ?? []} filterPlaceholder="Filter endpoints…" />;
}
