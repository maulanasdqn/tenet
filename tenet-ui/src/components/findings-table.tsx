import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import type { ColumnDef } from "@tanstack/react-table";
import { api, scanKeys } from "@/lib/api";
import type { Finding } from "@/lib/types";
import { DataTable } from "@/components/data-table";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { Confidence, SeverityBadge } from "@/components/status-badge";
import { DetailField } from "@/components/detail";

export function FindingsTable({ scanId }: { scanId: string }) {
  const query = useQuery({
    queryKey: scanKeys.findings(scanId),
    queryFn: () => api.getFindings(scanId),
  });

  const columns = useMemo<ColumnDef<Finding, unknown>[]>(
    () => [
      {
        accessorKey: "kind",
        header: "Kind",
        cell: ({ row }) => <Badge variant="outline">{row.original.kind}</Badge>,
      },
      { accessorKey: "name", header: "Name", cell: ({ row }) => <span className="font-medium">{row.original.name}</span> },
      {
        accessorKey: "value",
        header: "Value",
        cell: ({ row }) => <span className="text-muted-foreground">{row.original.value ?? "—"}</span>,
      },
      {
        accessorKey: "severity",
        header: "Severity",
        cell: ({ row }) => <SeverityBadge severity={row.original.severity} />,
      },
      {
        accessorKey: "confidence",
        header: "Confidence",
        cell: ({ row }) => <Confidence value={row.original.confidence} />,
      },
      {
        accessorKey: "evidence",
        header: "Evidence",
        cell: ({ row }) => (
          <span className="text-xs text-muted-foreground">{row.original.evidence ?? "—"}</span>
        ),
      },
    ],
    [],
  );

  if (query.isLoading) return <Skeleton className="h-64 w-full" />;
  if (query.isError) return <p className="text-sm text-destructive">{(query.error as Error).message}</p>;

  return (
    <DataTable
      columns={columns}
      data={query.data ?? []}
      filterPlaceholder="Filter findings…"
      renderDetail={(finding) => (
        <div className="flex flex-col">
          <div className="mb-2 flex items-center gap-2">
            <Badge variant="outline">{finding.kind}</Badge>
            <span className="font-medium">{finding.name}</span>
            <SeverityBadge severity={finding.severity} />
          </div>
          <DetailField label="Value">{finding.value ?? "—"}</DetailField>
          <DetailField label="Confidence">
            <Confidence value={finding.confidence} />
          </DetailField>
          <DetailField label="Evidence">
            <span className="text-muted-foreground">{finding.evidence ?? "—"}</span>
          </DetailField>
          <DetailField label="Discovered">
            {new Date(finding.created_at).toLocaleString()}
          </DetailField>
        </div>
      )}
    />
  );
}
