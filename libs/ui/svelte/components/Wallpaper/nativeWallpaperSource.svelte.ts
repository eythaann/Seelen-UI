import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { NativeWallpaperSnapshot } from "@seelen-ui/lib/types";
import { convertFileSrc } from "@tauri-apps/api/core";
import type { UnlistenFn } from "@tauri-apps/api/event";

// Shared source for the "Current Windows wallpaper" resource. The backend pushes a change event
// when Windows switches the wallpaper or overwrites the same file, so there is no polling here.

let imageSrc = $state("");
/** True when Windows has no wallpaper image or it can't be loaded, callers show a fallback. */
let unavailable = $state(false);

let users = 0;
let requested = "";
/** Bumped by every applied snapshot, so a stale initial fetch never overrides an event. */
let generation = 0;
let unlisten: Promise<UnlistenFn> | undefined;

function start(): void {
  // Listen first: an event may arrive while the initial snapshot is in flight.
  unlisten = subscribe(SeelenEvent.NativeShellWallpaperChanged, ({ payload }) => {
    apply(payload);
  });
  const initialGeneration = generation;
  invoke(SeelenCommand.GetNativeShellWallpaperSnapshot)
    .then((snapshot) => {
      if (generation === initialGeneration) apply(snapshot);
    })
    .catch((error) => console.error("Failed to read the Windows wallpaper", error));
}

function stop(): void {
  void unlisten?.then((fn) => fn());
  unlisten = undefined;
  requested = "";
  generation += 1;
}

function apply({ path, revision }: NativeWallpaperSnapshot): void {
  if (users === 0) return;
  generation += 1;

  if (!path) {
    requested = "";
    imageSrc = "";
    unavailable = true;
    return;
  }

  const next = `${convertFileSrc(path)}?revision=${revision}`;
  if (requested === next) return;
  requested = next;

  // Preload so the current image stays on screen until the next one is decoded.
  const image = new Image();
  image.onload = () => {
    if (users > 0 && requested === next) {
      imageSrc = next;
      unavailable = false;
    }
  };
  image.onerror = () => {
    if (requested !== next) return;
    console.error(`Failed to load the Windows wallpaper: ${path}`);
    requested = "";
    imageSrc = "";
    unavailable = true;
  };
  image.src = next;
}

class NativeWallpaperSource {
  get imageSrc() {
    return imageSrc;
  }
  get unavailable() {
    return unavailable;
  }

  /** Starts following the wallpaper while at least one user holds it, returns the release fn. */
  acquire(): () => void {
    users += 1;
    if (users === 1) start();
    return () => {
      users -= 1;
      if (users === 0) stop();
    };
  }
}

export const nativeWallpaperSource = new NativeWallpaperSource();
