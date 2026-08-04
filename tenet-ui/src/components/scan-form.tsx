import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Loader2, Radar } from "lucide-react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { rememberScan } from "@/lib/settings";
import type { Engine, TargetKind } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select } from "@/components/ui/select";

export function ScanForm() {
  const [target, setTarget] = useState("https://example.com");
  const [kind, setKind] = useState<TargetKind>("web");
  const [engine, setEngine] = useState<Engine>("browser");
  const [maxScripts, setMaxScripts] = useState("15");
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const mutation = useMutation({
    mutationFn: () =>
      api.createScan({
        target: target.trim(),
        kind,
        engine: kind === "mobile" ? "http" : engine,
        max_scripts: Number(maxScripts) || undefined,
      }),
    onSuccess: (response) => {
      const scan = response.data;
      rememberScan(scan);
      queryClient.invalidateQueries({ queryKey: ["scan"] });
      toast.success("Scan queued", { description: scan.scan_id });
      navigate({ to: "/scans/$scanId", params: { scanId: scan.scan_id } });
    },
    onError: (error: Error) => toast.error("Could not queue scan", { description: error.message }),
  });

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Radar className="size-4" /> New scan
        </CardTitle>
        <CardDescription>Point Tenet at a website or a mobile binary.</CardDescription>
      </CardHeader>
      <CardContent>
        <form
          className="flex flex-col gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            mutation.mutate();
          }}
        >
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="target">Target</Label>
            <Input
              id="target"
              value={target}
              onChange={(event) => setTarget(event.target.value)}
              placeholder="https://example.com or https://host/app.apk"
            />
          </div>
          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="kind">Kind</Label>
              <Select id="kind" value={kind} onChange={(event) => setKind(event.target.value as TargetKind)}>
                <option value="web">web</option>
                <option value="mobile">mobile</option>
              </Select>
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="engine">Engine</Label>
              <Select
                id="engine"
                value={engine}
                disabled={kind === "mobile"}
                onChange={(event) => setEngine(event.target.value as Engine)}
              >
                <option value="http">http</option>
                <option value="browser">browser</option>
              </Select>
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="max">Max scripts</Label>
              <Input
                id="max"
                type="number"
                min={1}
                max={100}
                value={maxScripts}
                onChange={(event) => setMaxScripts(event.target.value)}
              />
            </div>
          </div>
          <Button type="submit" disabled={mutation.isPending || !target.trim()}>
            {mutation.isPending ? <Loader2 className="size-4 animate-spin" /> : <Radar className="size-4" />}
            Queue scan
          </Button>
        </form>
      </CardContent>
    </Card>
  );
}
