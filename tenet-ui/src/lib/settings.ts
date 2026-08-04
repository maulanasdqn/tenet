const BASE_URL_KEY = "tenet.baseUrl";
const API_KEY_KEY = "tenet.apiKey";
const RECENT_KEY = "tenet.recentScans";
const RECENT_LIMIT = 40;

export const DEFAULT_BASE_URL = "http://127.0.0.1:8080";
export const DEFAULT_API_KEY = "dev-key";

export function getBaseUrl(): string {
  return localStorage.getItem(BASE_URL_KEY) ?? DEFAULT_BASE_URL;
}

export function getApiKey(): string {
  return localStorage.getItem(API_KEY_KEY) ?? DEFAULT_API_KEY;
}

export function saveSettings(baseUrl: string, apiKey: string) {
  localStorage.setItem(BASE_URL_KEY, baseUrl.trim().replace(/\/$/, ""));
  localStorage.setItem(API_KEY_KEY, apiKey.trim());
}

export interface RecentScan {
  scan_id: string;
  target: string;
  kind: string;
  engine: string;
  created_at: string;
}

export function recentScans(): RecentScan[] {
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    return raw ? (JSON.parse(raw) as RecentScan[]) : [];
  } catch {
    return [];
  }
}

export function rememberScan(scan: RecentScan) {
  const existing = recentScans().filter((entry) => entry.scan_id !== scan.scan_id);
  const next = [scan, ...existing].slice(0, RECENT_LIMIT);
  localStorage.setItem(RECENT_KEY, JSON.stringify(next));
}

export function forgetScan(scanId: string) {
  const next = recentScans().filter((entry) => entry.scan_id !== scanId);
  localStorage.setItem(RECENT_KEY, JSON.stringify(next));
}
