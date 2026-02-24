import { SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS, WallpaperConfiguration } from "@seelen-ui/lib";
import type { PlaybackSpeed, WallpaperId, WallpaperInstanceSettings } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { Wallpaper } from "libs/ui/react/components/Wallpaper/index.tsx";
import { Button, ColorPicker, Select, Slider, Switch } from "antd";
import { useTranslation } from "react-i18next";
import { useSearchParams } from "react-router";

import { patchWallpaperSettings, resetWallpaperSettings } from "./application.ts";
import { wallpapers } from "../../../state/resources.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";
import { ResourceDescription } from "../ResourceCard.tsx";

import styles from "./View.module.css";
import { settings } from "../../../state/mod.ts";

const playbackSpeeds: `${PlaybackSpeed}`[] = [
  "xDot25",
  "xDot5",
  "xDot75",
  "x1",
  "x1Dot25",
  "x1Dot5",
  "x1Dot75",
  "x2",
];
const playbackSpeedOptions = playbackSpeeds.map((s) => ({
  label: s.replace("Dot", ".").replace("x.", "x0."),
  value: s,
}));

const defaultWallpaperConfig = await WallpaperConfiguration.default();

export function SingleWallpaperView() {
  const [searchParams] = useSearchParams();
  const resourceId = searchParams.get("id") as WallpaperId;

  const editingWallpaper = wallpapers.value.find((wallpaper) => wallpaper.id === resourceId);

  const storedSettings = settings.value.byWallpaper;
  const config = {
    ...defaultWallpaperConfig,
    ...(storedSettings[resourceId] || {}),
  };

  const { t } = useTranslation();

  if (!editingWallpaper) {
    return <div>Ups 404</div>;
  }

  function patchWallpaperConfig(patch: Partial<WallpaperInstanceSettings>) {
    patchWallpaperSettings(resourceId, patch);
  }

  function onReset() {
    resetWallpaperSettings(resourceId);
  }

  return (
    <>
      <div className={styles.previewContainer}>
        <div className={styles.preview}>
          <Wallpaper definition={editingWallpaper} config={config} muted />
        </div>
      </div>

      <SettingsGroup>
        <b style={{ textAlign: "center", fontSize: "1.1rem" }}>
          <ResourceText text={editingWallpaper.metadata.displayName} />
        </b>
        <ResourceDescription text={editingWallpaper.metadata.description} />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("reset_all_to_default")}
          action={
            <Button onClick={onReset}>
              <Icon iconName="RiResetLeftLine" />
            </Button>
          }
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("wall.playback")}
          action={
            <Select
              value={config.playbackSpeed}
              options={playbackSpeedOptions}
              onSelect={(playbackSpeed) => {
                patchWallpaperConfig({ playbackSpeed });
              }}
            />
          }
        />

        <SettingsOption
          label={t("wall.flipHorizontal")}
          action={
            <Switch
              value={config.flipHorizontal}
              onChange={(flipHorizontal) => {
                patchWallpaperConfig({ flipHorizontal });
              }}
            />
          }
        />

        <SettingsOption
          label={t("wall.flipVertical")}
          action={
            <Switch
              value={config.flipVertical}
              onChange={(flipVertical) => {
                patchWallpaperConfig({ flipVertical });
              }}
            />
          }
        />

        <SettingsOption
          label={t("wall.blur")}
          action={
            <Slider
              value={config.blur}
              min={0}
              max={50}
              onChange={(blur) => {
                patchWallpaperConfig({ blur });
              }}
            />
          }
        />

        <SettingsOption
          label={t("wall.objectFit")}
          action={
            <Select
              value={config.objectFit}
              options={[
                { label: t("wall.fit.cover"), value: "cover" },
                { label: t("wall.fit.contain"), value: "contain" },
                { label: t("wall.fit.fill"), value: "fill" },
              ]}
              onSelect={(objectFit) => {
                patchWallpaperConfig({ objectFit });
              }}
            />
          }
        />

        <SettingsOption
          label={t("wall.objectPosition")}
          action={
            <Select
              value={config.objectPosition}
              options={[
                { label: t("wall.position.top"), value: "top" },
                { label: t("wall.position.center"), value: "center" },
                { label: t("wall.position.bottom"), value: "bottom" },
                { label: t("wall.position.left"), value: "left" },
                { label: t("wall.position.right"), value: "right" },
              ]}
              onSelect={(objectPosition) => {
                patchWallpaperConfig({ objectPosition });
              }}
            />
          }
        />

        <SettingsOption
          label={t("wall.saturation")}
          action={
            <Slider
              value={config.saturation}
              min={0}
              step={0.01}
              max={2}
              onChange={(saturation) => {
                patchWallpaperConfig({ saturation });
              }}
            />
          }
        />

        <SettingsOption
          label={t("wall.contrast")}
          action={
            <Slider
              value={config.contrast}
              min={0}
              step={0.01}
              max={2}
              onChange={(contrast) => {
                patchWallpaperConfig({ contrast });
              }}
            />
          }
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption
              label={t("wall.withOverlay")}
              action={
                <Switch
                  value={config.withOverlay}
                  onChange={(withOverlay) => {
                    patchWallpaperConfig({ withOverlay });
                  }}
                />
              }
            />
          }
        >
          <SettingsOption
            label={t("wall.overlayMixBlendMode")}
            action={
              <Select
                value={config.overlayMixBlendMode}
                options={[
                  { label: "normal", value: "normal" },
                  { label: "multiply", value: "multiply" },
                  { label: "screen", value: "screen" },
                  { label: "overlay", value: "overlay" },
                  { label: "darken", value: "darken" },
                  { label: "lighten", value: "lighten" },
                  { label: "color-dodge", value: "color-dodge" },
                  { label: "color-burn", value: "color-burn" },
                  { label: "hard-light", value: "hard-light" },
                  { label: "soft-light", value: "soft-light" },
                  { label: "difference", value: "difference" },
                  { label: "exclusion", value: "exclusion" },
                  { label: "hue", value: "hue" },
                  { label: "saturation", value: "saturation" },
                  { label: "color", value: "color" },
                  { label: "luminosity", value: "luminosity" },
                  { label: "plus-darker", value: "plus-darker" },
                  { label: "plus-lighter", value: "plus-lighter" },
                ]}
                onSelect={(overlayMixBlendMode) => {
                  patchWallpaperConfig({ overlayMixBlendMode });
                }}
              />
            }
          />

          <SettingsOption
            label={t("wall.overlayColor")}
            action={
              <ColorPicker
                showText
                value={config.overlayColor}
                onChangeComplete={(color) => {
                  patchWallpaperConfig({ overlayColor: color.toHexString() });
                }}
              />
            }
          />
        </SettingsSubGroup>
      </SettingsGroup>

      {SUPPORTED_VIDEO_WALLPAPER_EXTENSIONS.some((ext) => editingWallpaper.filename?.toLowerCase()?.endsWith(ext)) && (
        <SettingsGroup>
          <SettingsOption
            label={t("wall.muted")}
            action={
              <Switch
                value={config.muted}
                onChange={(muted) => {
                  patchWallpaperConfig({ muted });
                }}
              />
            }
          />
        </SettingsGroup>
      )}
    </>
  );
}
