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

  return ensureDeviceAvailable({ view: "mixer", deviceId }, availableDeviceIds);
}

/** Falls back to the overview when a mixer view points at a device that is no longer available. */
export function ensureDeviceAvailable(value: LastView, availableDeviceIds: Iterable<string>): LastView {
  if (value.view !== "mixer") return value;
  if (!value.deviceId || !new Set(availableDeviceIds).has(value.deviceId)) return DEFAULT_VIEW;
  return value;
}

export function serializeLastView(value: LastView): string {
  return JSON.stringify(value.view === "mixer" && value.deviceId ? value : DEFAULT_VIEW);
}
