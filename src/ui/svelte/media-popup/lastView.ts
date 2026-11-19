export type MediaPopupView = "main" | "mixer";

export interface LastView {
  view: MediaPopupView;
  deviceId: string | null;
}

export const DEFAULT_VIEW: LastView = { view: "main", deviceId: null };

/**
 * Parses the stored last view. Falls back to the overview when the value is missing or invalid,
 * or when the remembered device is no longer available (unplugged, disabled, driver change...).
 */
export function resolveLastView(stored: string | null, availableDeviceIds: Iterable<string>): LastView {
  if (!stored) return DEFAULT_VIEW;

  let parsed: unknown;
  try {
    parsed = JSON.parse(stored);
  } catch {
    return DEFAULT_VIEW;
  }

  if (typeof parsed !== "object" || parsed === null) return DEFAULT_VIEW;
  const { view, deviceId } = parsed as Record<string, unknown>;

  if (view !== "mixer" || typeof deviceId !== "string") return DEFAULT_VIEW;
  if (!new Set(availableDeviceIds).has(deviceId)) return DEFAULT_VIEW;

  return { view: "mixer", deviceId };
}

export function serializeLastView(value: LastView): string {
  return JSON.stringify(value.view === "mixer" && value.deviceId ? value : DEFAULT_VIEW);
}
