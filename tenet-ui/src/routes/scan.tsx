import { createRoute } from "@tanstack/react-router";
import { ScanDetail } from "@/components/scan-detail";
import { rootRoute } from "./root";

export const scanRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/scans/$scanId",
  component: ScanRoute,
});

function ScanRoute() {
  const { scanId } = scanRoute.useParams();
  return <ScanDetail scanId={scanId} />;
}
