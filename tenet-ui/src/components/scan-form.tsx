import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useForm } from "@tanstack/react-form";
import { useNavigate } from "@tanstack/react-router";
import { Loader2, Radar } from "lucide-react";
import { toast } from "sonner";
import { api } from "@/lib/api";
import { rememberScan } from "@/lib/store";
import { createScanSchema, type CreateScanValues } from "@/lib/schemas";
import type { Engine, TargetKind } from "@/lib/types";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { FieldError } from "@/components/field-error";

export function ScanForm() {
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const mutation = useMutation({
    mutationFn: (value: CreateScanValues) =>
      api.createScan({
        target: value.target.trim(),
        kind: value.kind,
        engine: value.kind === "mobile" ? "http" : value.engine,
        max_scripts: value.maxScripts,
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

  const form = useForm({
    defaultValues: {
      target: "https://example.com",
      kind: "web" as TargetKind,
      engine: "browser" as Engine,
      maxScripts: 15,
    },
    validators: { onChange: createScanSchema },
    onSubmit: async ({ value }) => {
      await mutation.mutateAsync(value);
    },
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
            event.stopPropagation();
            form.handleSubmit();
          }}
        >
          <form.Field name="target">
            {(field) => (
              <div className="flex flex-col gap-1.5">
                <Label htmlFor="target">Target</Label>
                <Input
                  id="target"
                  value={field.state.value}
                  onBlur={field.handleBlur}
                  onChange={(event) => field.handleChange(event.target.value)}
                  placeholder="https://example.com or https://host/app.apk"
                />
                <FieldError meta={field.state.meta} />
              </div>
            )}
          </form.Field>

          <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
            <form.Field name="kind">
              {(field) => (
                <div className="flex flex-col gap-1.5">
                  <Label htmlFor="kind">Kind</Label>
                  <Select
                    value={field.state.value}
                    onValueChange={(value) => field.handleChange(value as TargetKind)}
                  >
                    <SelectTrigger id="kind">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="web">web</SelectItem>
                      <SelectItem value="mobile">mobile</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              )}
            </form.Field>

            <form.Subscribe selector={(state) => state.values.kind}>
              {(kind) => (
                <form.Field name="engine">
                  {(field) => (
                    <div className="flex flex-col gap-1.5">
                      <Label htmlFor="engine">Engine</Label>
                      <Select
                        value={field.state.value}
                        disabled={kind === "mobile"}
                        onValueChange={(value) => field.handleChange(value as Engine)}
                      >
                        <SelectTrigger id="engine">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="http">http</SelectItem>
                          <SelectItem value="browser">browser</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  )}
                </form.Field>
              )}
            </form.Subscribe>

            <form.Field name="maxScripts">
              {(field) => (
                <div className="flex flex-col gap-1.5">
                  <Label htmlFor="max">Max scripts</Label>
                  <Input
                    id="max"
                    type="number"
                    min={1}
                    max={100}
                    value={field.state.value}
                    onBlur={field.handleBlur}
                    onChange={(event) => field.handleChange(Number(event.target.value))}
                  />
                  <FieldError meta={field.state.meta} />
                </div>
              )}
            </form.Field>
          </div>

          <form.Subscribe selector={(state) => [state.canSubmit, state.isSubmitting] as const}>
            {([canSubmit, isSubmitting]) => (
              <Button type="submit" disabled={!canSubmit || isSubmitting || mutation.isPending}>
                {isSubmitting || mutation.isPending ? (
                  <Loader2 className="size-4 animate-spin" />
                ) : (
                  <Radar className="size-4" />
                )}
                Queue scan
              </Button>
            )}
          </form.Subscribe>
        </form>
      </CardContent>
    </Card>
  );
}
