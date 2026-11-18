import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { NativeWallpaperSnapshot } from "@seelen-ui/lib/types";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useState } from "preact/hooks";

import type { DefinedWallProps } from "../types";
import { getWallpaperStyles } from "../utils.ts";
import { cx } from "libs/ui/react/utils/styling.ts";
import { ThemedWallpaper } from "./ThemedWallpaper";
import cs from "../index.module.css";

export function NativeWallpaper({ definition, config, onLoad }: DefinedWallProps) {
  const [imageSrc, setImageSrc] = useState("");
  // True when Windows has no wallpaper image or it cannot be loaded.
  const [unavailable, setUnavailable] = useState(false);

  useEffect(() => {
    let cancelled = false;
    let requested = "";
    // Bumped by every applied snapshot, so a stale initial fetch never overrides an event.
    let generation = 0;

    function apply({ path, revision }: NativeWallpaperSnapshot) {
      if (cancelled) return;
      generation += 1;

      if (!path) {
        requested = "";
        setImageSrc("");
        setUnavailable(true);
        return;
      }

      const next = `${convertFileSrc(path)}?revision=${revision}`;
      if (requested === next) return;
      requested = next;

      // Preload so the current image stays on screen until the next one is decoded.
      const image = new Image();
      image.onload = () => {
        if (!cancelled && requested === next) {
          setImageSrc(next);
          setUnavailable(false);
        }
      };
      image.onerror = () => {
        if (cancelled || requested !== next) return;
        console.error(`Failed to load the Windows wallpaper: ${path}`);
        requested = "";
        setImageSrc("");
        setUnavailable(true);
      };
      image.src = next;
    }

    // Listen first: an event may arrive while the initial snapshot is in flight.
    const unlisten = subscribe(SeelenEvent.NativeShellWallpaperChanged, ({ payload }) => {
      apply(payload);
    });
    const initialGeneration = generation;
    invoke(SeelenCommand.GetNativeShellWallpaperSnapshot)
      .then((snapshot) => {
        if (generation === initialGeneration) apply(snapshot);
      })
      .catch((error) => console.error("Failed to read the Windows wallpaper", error));

    return () => {
      cancelled = true;
      void unlisten.then((fn) => fn());
    };
  }, []);

  if (imageSrc) {
    return (
      <img
        id={definition.id}
        className={cx(cs.wallpaper, "wallpaper")}
        style={getWallpaperStyles(config)}
        src={imageSrc}
        onLoad={onLoad}
        decoding="async"
        alt=""
      />
    );
  }

  // Solid color background or unreadable file: show the default wallpaper instead of nothing.
  return unavailable ? <ThemedWallpaper onLoad={onLoad} /> : null;
}
