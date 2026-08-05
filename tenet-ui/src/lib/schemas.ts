import { z } from "zod";

function isUrl(value: string): boolean {
  try {
    new URL(value);
    return true;
  } catch {
    return false;
  }
}

export const createScanSchema = z
  .object({
    target: z.string().min(1, "Target is required").max(2048),
    kind: z.enum(["web", "mobile"]),
    engine: z.enum(["http", "browser"]),
    maxScripts: z.number().int().min(1, "At least 1").max(100, "At most 100"),
  })
  .refine((value) => value.kind === "mobile" || /^https?:\/\//i.test(value.target.trim()), {
    message: "Web targets must start with http:// or https://",
    path: ["target"],
  });

export type CreateScanValues = z.infer<typeof createScanSchema>;

export const settingsSchema = z.object({
  baseUrl: z.string().min(1, "Gateway URL is required").refine(isUrl, "Enter a valid URL"),
  apiKey: z.string().min(1, "API key is required"),
});

export type SettingsValues = z.infer<typeof settingsSchema>;
