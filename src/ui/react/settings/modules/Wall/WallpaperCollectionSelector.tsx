import { Select } from "antd";
import type { CSSProperties } from "react";
import { useTranslation } from "react-i18next";

import { ResourceKind, type WallpaperId } from "@seelen-ui/lib/types";
import { wallpapers } from "../../state/resources.ts";
import { resolveDisplayName, ResourcePortrait } from "../resources/ResourceCard.tsx";
import { getOrCreateHiddenCollection, getWallpaperCollections } from "./application.ts";
import type { DefaultOptionType } from "antd/es/select/index";

import cs from "./WallpaperCollectionSelector.module.css";

interface Props {
  value: string | null | undefined;
  onChange: (collectionId: string | null) => void;
  style?: CSSProperties;
  placeholder?: string;
}

const WALLPAPER_PREFIX = "wallpaper:";

export function WallpaperCollectionSelector({ value, onChange, style, placeholder }: Props) {
  const { t, i18n } = useTranslation();

  const allCollections = getWallpaperCollections();
  const visibleCollections = allCollections.filter((c) => !c.hidden);
  const allWallpapers = wallpapers.value;

  // If the selected collection is a hidden single-wallpaper collection, show the wallpaper as selected
  const currentHidden = value ? allCollections.find((c) => c.id === value && c.hidden) : null;
  const displayValue = currentHidden && currentHidden.wallpapers.length === 1
    ? `${WALLPAPER_PREFIX}${currentHidden.wallpapers[0]}`
    : (value ?? undefined);

  function handleChange(v: string | undefined) {
    if (!v) {
      onChange(null);
      return;
    }
    if (v.startsWith(WALLPAPER_PREFIX)) {
      const wallpaperId = v.slice(WALLPAPER_PREFIX.length) as WallpaperId;
      const wp = allWallpapers.find((w) => w.id === wallpaperId);
      const name = wp ? resolveDisplayName(wp.metadata.displayName, i18n.language) : wallpaperId;
      onChange(getOrCreateHiddenCollection(wallpaperId, name));
    } else {
      onChange(v);
    }
  }

  let options: DefaultOptionType[] = [];

  if (visibleCollections.length > 0) {
    options.push({
      label: t("wall.collections"),
      options: visibleCollections.map((c) => ({ label: c.name, value: c.id, name: c.name })),
    });
  }

  if (allWallpapers.length > 0) {
    options.push({
      label: t("wall.wallpapers"),
      options: allWallpapers.map((w) => {
        const name = resolveDisplayName(w.metadata.displayName, i18n.language);
        return {
          name,
          label: (
            <div className={cs.entry}>
              <ResourcePortrait resource={w} kind={ResourceKind.Wallpaper} />
              <span className={cs.label}>{name}</span>
            </div>
          ),
          value: `${WALLPAPER_PREFIX}${w.id}`,
        };
      }),
    });
  }

  return (
    <Select
      style={style}
      value={displayValue}
      onChange={handleChange}
      placeholder={placeholder}
      allowClear
      options={options}
      showSearch={{
        optionFilterProp: "name",
      }}
    />
  );
}
