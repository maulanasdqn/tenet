import { useState } from "react";
import { useForm } from "@tanstack/react-form";
import { Settings } from "lucide-react";
import { toast } from "sonner";
import { saveSettings, settingsStore } from "@/lib/store";
import { settingsSchema } from "@/lib/schemas";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { FieldError } from "@/components/field-error";

export function SettingsPanel() {
  const [open, setOpen] = useState(false);

  const form = useForm({
    defaultValues: { ...settingsStore.state },
    validators: { onChange: settingsSchema },
    onSubmit: ({ value }) => {
      saveSettings(value);
      toast.success("Settings saved");
      setOpen(false);
    },
  });

  return (
    <>
      <Button variant="outline" size="sm" onClick={() => setOpen(true)}>
        <Settings className="size-4" /> Settings
      </Button>
      {open ? (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4"
          onClick={() => setOpen(false)}
        >
          <Card className="w-full max-w-md" onClick={(event) => event.stopPropagation()}>
            <CardHeader>
              <CardTitle>Gateway connection</CardTitle>
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
                <form.Field name="baseUrl">
                  {(field) => (
                    <div className="flex flex-col gap-1.5">
                      <Label htmlFor="baseUrl">Gateway URL</Label>
                      <Input
                        id="baseUrl"
                        value={field.state.value}
                        onBlur={field.handleBlur}
                        onChange={(event) => field.handleChange(event.target.value)}
                      />
                      <FieldError meta={field.state.meta} />
                    </div>
                  )}
                </form.Field>
                <form.Field name="apiKey">
                  {(field) => (
                    <div className="flex flex-col gap-1.5">
                      <Label htmlFor="apiKey">API key</Label>
                      <Input
                        id="apiKey"
                        type="password"
                        value={field.state.value}
                        onBlur={field.handleBlur}
                        onChange={(event) => field.handleChange(event.target.value)}
                      />
                      <FieldError meta={field.state.meta} />
                    </div>
                  )}
                </form.Field>
                <div className="flex justify-end gap-2">
                  <Button type="button" variant="ghost" onClick={() => setOpen(false)}>
                    Cancel
                  </Button>
                  <form.Subscribe selector={(state) => state.canSubmit}>
                    {(canSubmit) => (
                      <Button type="submit" disabled={!canSubmit}>
                        Save
                      </Button>
                    )}
                  </form.Subscribe>
                </div>
              </form>
            </CardContent>
          </Card>
        </div>
      ) : null}
    </>
  );
}
