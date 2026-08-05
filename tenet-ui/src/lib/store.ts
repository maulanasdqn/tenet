import { Store } from "@tanstack/store";

export const DEFAULT_BASE_URL = "http://127.0.0.1:8080";
export const DEFAULT_API_KEY = "dev-key";

const SETTINGS_KEY = "tenet.settings";
const RECENT_KEY = "tenet.recentScans";
const RECENT_LIMIT = 40;

export interface Settings {
  baseUrl: string;
  apiKey: string;
}

export interface RecentScan {
  scan_id: string;
  target: string;
  kind: string;
  engine: string;
  created_at: string;
}

function load<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : fallback;
  } catch {
    return fallback;
  }
}

export const settingsStore = new Store<Settings>(
  load(SETTINGS_KEY, { baseUrl: DEFAULT_BASE_URL, apiKey: DEFAULT_API_KEY }),
);

export const recentScansStore = new Store<RecentScan[]>(load(RECENT_KEY, []));

settingsStore.subscribe(() => localStorage.setItem(SETTINGS_KEY, JSON.stringify(settingsStore.state)));
recentScansStore.subscribe(() => localStorage.setItem(RECENT_KEY, JSON.stringify(recentScansStore.state)));

export function saveSettings(next: Settings) {
  settingsStore.setState(() => ({
    baseUrl: next.baseUrl.trim().replace(/\/$/, ""),
    apiKey: next.apiKey.trim(),
  }));
}

export function rememberScan(scan: RecentScan) {
  recentScansStore.setState((prev) =>
    [scan, ...prev.filter((entry) => entry.scan_id !== scan.scan_id)].slice(0, RECENT_LIMIT),
  );
}

export function forgetScan(scanId: string) {
  recentScansStore.setState((prev) => prev.filter((entry) => entry.scan_id !== scanId));
}
