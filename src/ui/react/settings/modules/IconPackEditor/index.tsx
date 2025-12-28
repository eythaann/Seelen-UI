import type { Icon, IconPackEntry } from "@seelen-ui/lib/types";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Input } from "antd";
import { useMemo, useState } from "react";
import { useSelector } from "react-redux";

import { newSelectors } from "../shared/store/app/reducer.ts";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";
import cs from "./index.module.css";

function resolveAsSrc(parent: string, icon: Icon): Icon {
  return {
    base: icon.base ? convertFileSrc(`${parent}\\${icon.base}`) : null,
    light: icon.light ? convertFileSrc(`${parent}\\${icon.light}`) : null,
    dark: icon.dark ? convertFileSrc(`${parent}\\${icon.dark}`) : null,
    mask: icon.mask ? convertFileSrc(`${parent}\\${icon.mask}`) : null,
    isAproximatelySquare: icon.isAproximatelySquare,
  };
}

export function IconPackEditorView() {
  const [filterValue, setFilterValue] = useState("");
  const iconPacks = useSelector(newSelectors.availableIconPacks);

  const entries = useMemo(() => {
    const system = iconPacks.find((i) => i.id === "@system/icon-pack");
    return (
      system?.entries
        .filter((e) => containsSearched(e, filterValue))
        .map((e) => {
          const newEntry = { ...e };
          if (newEntry.icon) {
            newEntry.icon = resolveAsSrc(system.metadata.path, newEntry.icon);
          }
          return newEntry;
        }) || []
    );
  }, [iconPacks, filterValue]);

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>Search</b>
          <Input
            value={filterValue}
            onChange={(e) => setFilterValue(e.currentTarget.value)}
            placeholder="example: discord..."
          />
        </SettingsOption>
      </SettingsGroup>
      {entries.map((entry, idx) => {
        if (!entry.icon) {
          return null;
        }

        return (
          <SettingsGroup key={idx}>
            <IconTitle entry={entry} />
            <IconEditor icon={entry.icon} />
          </SettingsGroup>
        );
      })}
    </>
  );
}

function IconTitle({ entry }: { entry: IconPackEntry }) {
  if (entry.type === "unique") {
    return (
      <h3>
        <p>
          <b>umid:</b>
          {entry.umid}
        </p>
        <p>
          <b>path or filename:</b>
          {entry.path}
        </p>
      </h3>
    );
  }

  if (entry.type === "shared") {
    return (
      <h3>
        <b>Extension:</b>
        {entry.extension}
      </h3>
    );
  }

  return (
    <h3>
      <b>Key:</b>
      {entry.key}
    </h3>
  );
}

function IconEditor({ icon }: { icon: Icon }) {
  return (
    <div className={cs.iconGroup}>
      {!!icon.base && (
        <div className={cs.iconContainer}>
          <div className={cs.iconBox}>
            <img src={icon.base} loading="lazy" />
          </div>
          <span className={cs.iconLabel}>base</span>
        </div>
      )}
      {!!icon.light && (
        <div className={cs.iconContainer}>
          <div className={cs.iconBox}>
            <img src={icon.light} loading="lazy" />
          </div>
          <span className={cs.iconLabel}>light</span>
        </div>
      )}
      {!!icon.dark && (
        <div className={cs.iconContainer}>
          <div className={cs.iconBox}>
            <img src={icon.dark} loading="lazy" />
          </div>
          <span className={cs.iconLabel}>dark</span>
        </div>
      )}
      {!!icon.mask && (
        <div className={cs.iconContainer}>
          <div className={cs.iconBox}>
            <img src={icon.mask} loading="lazy" />
          </div>
          <span className={cs.iconLabel}>mask</span>
        </div>
      )}
    </div>
  );
}

function containsSearched(entry: IconPackEntry, filterValue: string) {
  const searchString = filterValue.toLowerCase();
  return (
    (entry.type === "unique" &&
      (!!entry.path?.toLowerCase().includes(searchString) ||
        !!entry.umid?.toLowerCase().includes(searchString))) ||
    (entry.type === "shared" && entry.extension.toLowerCase().includes(searchString)) ||
    (entry.type === "custom" && entry.key.toLowerCase().includes(searchString))
  );
}
