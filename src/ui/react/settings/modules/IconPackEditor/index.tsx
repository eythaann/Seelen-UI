import type { Icon, IconPackEntry } from "@seelen-ui/lib/types";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Button, Input, message } from "antd";
import { useMemo, useState } from "react";

import { iconPacks } from "../../state/resources.ts";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";
import cs from "./index.module.css";
import { Icon as ReactIcon } from "libs/ui/react/components/Icon/index.tsx";

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

  const entries = useMemo(() => {
    const system = iconPacks.value.find((i) => i.id === "@system/icon-pack");
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
  }, [filterValue]);

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
  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard
      .writeText(text)
      .then(() => {
        message.success(`${label} copied to clipboard`);
      })
      .catch(() => {
        message.error(`Failed to copy ${label}`);
      });
  };

  if (entry.type === "unique") {
    return (
      <div>
        {entry.umid && (
          <p>
            <b>umid:</b>
            {entry.umid}
            <Button
              type="text"
              size="small"
              onClick={() => copyToClipboard(entry.umid!, "UMID")}
              style={{ marginLeft: 8 }}
            >
              <ReactIcon iconName="IoCopyOutline" />
            </Button>
          </p>
        )}
        {entry.path && (
          <p>
            <b>path or filename:</b>
            {entry.path}
            <Button
              type="text"
              onClick={() => copyToClipboard(entry.path!, "Path")}
              style={{ marginLeft: 8 }}
            >
              <ReactIcon iconName="IoCopyOutline" />
            </Button>
          </p>
        )}
      </div>
    );
  }

  if (entry.type === "shared") {
    return (
      <div>
        <b>Extension:</b>
        {entry.extension}
        <Button
          type="text"
          size="small"
          onClick={() => copyToClipboard(entry.extension, "Extension")}
          style={{ marginLeft: 8 }}
        >
          <ReactIcon iconName="IoCopyOutline" />
        </Button>
      </div>
    );
  }

  return (
    <div>
      <b>Key:</b>
      {entry.key}
      <Button
        type="text"
        size="small"
        onClick={() => copyToClipboard(entry.key, "Key")}
        style={{ marginLeft: 8 }}
      >
        <ReactIcon iconName="IoCopyOutline" />
      </Button>
    </div>
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
