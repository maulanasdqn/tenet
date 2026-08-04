import { getApiKey, getBaseUrl } from "./settings";
import type {
  CreateScanInput,
  EndpointRow,
  Finding,
  ListResponse,
  Scan,
  ScanCreated,
  SingleResponse,
} from "./types";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${getBaseUrl()}${path}`, {
    ...init,
    headers: {
      "content-type": "application/json",
      "x-api-key": getApiKey(),
      ...(init?.headers ?? {}),
    },
  });
  if (!response.ok) {
    const message = await response.text().catch(() => response.statusText);
    throw new Error(message || `request failed with ${response.status}`);
  }
  return (await response.json()) as T;
}

export const api = {
  health(): Promise<Response> {
    return fetch(`${getBaseUrl()}/healthz`);
  },
  createScan(input: CreateScanInput): Promise<SingleResponse<ScanCreated>> {
    return request("/v1/scans", { method: "POST", body: JSON.stringify(input) });
  },
  getScan(id: string): Promise<Scan> {
    return request<SingleResponse<Scan>>(`/v1/scans/${id}`).then((r) => r.data);
  },
  getFindings(id: string): Promise<Finding[]> {
    return request<ListResponse<Finding>>(`/v1/scans/${id}/findings`).then((r) => r.data);
  },
  getEndpoints(id: string): Promise<EndpointRow[]> {
    return request<ListResponse<EndpointRow>>(`/v1/scans/${id}/endpoints`).then((r) => r.data);
  },
  getOpenapi(id: string): Promise<unknown> {
    return request<unknown>(`/v1/scans/${id}/openapi`);
  },
};

export const scanKeys = {
  detail: (id: string) => ["scan", id] as const,
  findings: (id: string) => ["scan", id, "findings"] as const,
  endpoints: (id: string) => ["scan", id, "endpoints"] as const,
  openapi: (id: string) => ["scan", id, "openapi"] as const,
};
