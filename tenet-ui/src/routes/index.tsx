import { createRoute } from "@tanstack/react-router";
import { ScanForm } from "@/components/scan-form";
import { RecentScans } from "@/components/recent-scans";
import { rootRoute } from "./root";

export const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  component: Dashboard,
});

function Dashboard() {
  return (
    <div className="flex flex-col gap-6">
      <ScanForm />
      <RecentScans />
    </div>
  );
}
