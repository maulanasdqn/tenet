import { Link, Outlet, createRootRoute } from "@tanstack/react-router";
import { Radar } from "lucide-react";
import { SettingsPanel } from "@/components/settings-panel";

export const rootRoute = createRootRoute({
  component: RootLayout,
});

function RootLayout() {
  return (
    <div className="min-h-screen">
      <header className="sticky top-0 z-40 border-b border-border bg-background/80 backdrop-blur">
        <div className="mx-auto flex max-w-5xl items-center justify-between px-4 py-3">
          <Link to="/" className="flex items-center gap-2 font-semibold">
            <Radar className="size-5 text-sky-400" />
            <span>Tenet</span>
            <span className="text-xs font-normal text-muted-foreground">reverse engineering</span>
          </Link>
          <SettingsPanel />
        </div>
      </header>
      <main className="mx-auto max-w-5xl px-4 py-8">
        <Outlet />
      </main>
    </div>
  );
}
