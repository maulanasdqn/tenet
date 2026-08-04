export type TargetKind = "web" | "mobile";
export type Engine = "http" | "browser";

export interface CreateScanInput {
  target: string;
  kind: TargetKind;
  engine: Engine;
  max_scripts?: number;
}

export interface ScanCreated {
  scan_id: string;
  target: string;
  kind: string;
  engine: string;
  status: string;
  created_at: string;
}

export interface Scan {
  scan_id: string;
  target: string;
  kind: string;
  engine: string;
  status: "queued" | "running" | "succeeded" | "failed";
  attempts: number;
  max_attempts: number;
  error: string | null;
  created_at: string;
  started_at: string | null;
  finished_at: string | null;
}

export interface Finding {
  kind: string;
  name: string;
  value: string | null;
  severity: string;
  confidence: number;
  evidence: string | null;
  created_at: string;
}

export interface EndpointRow {
  method: string;
  path: string;
  base_url: string | null;
  source: string;
  confidence: number;
  created_at: string;
}

export interface SingleResponse<T> {
  data: T;
  message: string;
}

export interface ListResponse<T> {
  data: T[];
  total: number;
}
