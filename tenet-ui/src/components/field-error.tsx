interface FieldMeta {
  isTouched: boolean;
  errors: unknown[];
}

function message(error: unknown): string {
  if (typeof error === "string") return error;
  if (error && typeof error === "object" && "message" in error) {
    return String((error as { message: unknown }).message);
  }
  return "Invalid";
}

export function FieldError({ meta }: { meta: FieldMeta }) {
  if (!meta.isTouched || meta.errors.length === 0) return null;
  return <p className="text-xs text-destructive">{message(meta.errors[0])}</p>;
}
