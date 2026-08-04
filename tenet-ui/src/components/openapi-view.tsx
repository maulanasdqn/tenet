import { useQuery } from "@tanstack/react-query";
import { Download } from "lucide-react";
import { api, scanKeys } from "@/lib/api";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";

export function OpenapiView({ scanId }: { scanId: string }) {
  const query = useQuery({
    queryKey: scanKeys.openapi(scanId),
    queryFn: () => api.getOpenapi(scanId),
  });

  if (query.isLoading) return <Skeleton className="h-64 w-full" />;
  if (query.isError) return <p className="text-sm text-destructive">{(query.error as Error).message}</p>;

  const text = JSON.stringify(query.data, null, 2);
  const download = () => {
    const blob = new Blob([text], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `openapi-${scanId}.json`;
    anchor.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div className="flex flex-col gap-3">
      <div>
        <Button variant="outline" size="sm" onClick={download}>
          <Download className="size-4" /> Download openapi.json
        </Button>
      </div>
      <pre className="max-h-[32rem] overflow-auto rounded-lg border border-border bg-muted/30 p-4 text-xs">
        {text}
      </pre>
    </div>
  );
}
