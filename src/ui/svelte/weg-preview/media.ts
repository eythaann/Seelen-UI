interface WindowIdentity {
  umid: string | null;
  process: { path: string | null };
}

export function executableName(path: string | null): string {
  return path?.replaceAll("\\", "/").split("/").pop()?.toLowerCase() ?? "";
}

export function mediaShortcutProfile<T>(profiles: Record<string, T>, exe: string): T | undefined {
  const name = exe.toLowerCase();
  return profiles[name] ?? Object.entries(profiles).find(([key]) => key.toLowerCase() === name)?.[1];
}

/** Never pick the system default or match a window title: those may belong to another app. */
export function matchingSessions<T extends { umid: string }>(window: WindowIdentity, sessions: T[]): T[] {
  if (window.umid) {
    const exact = sessions.filter((session) => session.umid.toLowerCase() === window.umid!.toLowerCase());
    if (exact.length) return exact;
  }
  const exe = executableName(window.process.path);
  if (!exe.endsWith(".exe")) return [];
  return sessions.filter((session) => executableName(session.umid) === exe);
}

/** Keep in sync with the backend's deliberately small virtual-key chord grammar. */
export function isMediaShortcut(value: string): boolean {
  const parts = value.toUpperCase().split("+").map((part) => part.trim());
  if (parts.length < 2 || parts.length > 4) return false;
  const modifiers = parts.slice(0, -1).map((part) => part === "CONTROL" ? "CTRL" : part);
  if (modifiers.some((part) => !["CTRL", "ALT", "SHIFT"].includes(part))) return false;
  if (new Set(modifiers).size !== modifiers.length) return false;
  return /^(?:[A-Z0-9=-]|SPACE|LEFT|UP|RIGHT|DOWN|F(?:[1-9]|1\d|2[0-4]))$/.test(parts.at(-1)!);
}

export const knownPlayers = new Set(["cloudmusic.exe", "qqmusic.exe"]);
export const browsers = new Set(["msedge.exe", "chrome.exe", "firefox.exe", "brave.exe", "vivaldi.exe", "opera.exe"]);
