import { Badge } from "@/components/ui/badge";

const STATUS_VARIANT: Record<string, "info" | "warning" | "success" | "danger"> = {
  queued: "info",
  running: "warning",
  succeeded: "success",
  failed: "danger",
};

const SEVERITY_VARIANT: Record<string, "secondary" | "info" | "warning" | "danger"> = {
  info: "secondary",
  low: "info",
  medium: "warning",
  high: "danger",
  critical: "danger",
};

export function StatusBadge({ status }: { status: string }) {
  return <Badge variant={STATUS_VARIANT[status] ?? "secondary"}>{status}</Badge>;
}

export function SeverityBadge({ severity }: { severity: string }) {
  return <Badge variant={SEVERITY_VARIANT[severity] ?? "secondary"}>{severity}</Badge>;
}

export function Confidence({ value }: { value: number }) {
  const pct = Math.round(value * 100);
  const tone = value >= 0.8 ? "text-emerald-400" : value >= 0.5 ? "text-amber-400" : "text-muted-foreground";
  return <span className={tone}>{pct}%</span>;
}
