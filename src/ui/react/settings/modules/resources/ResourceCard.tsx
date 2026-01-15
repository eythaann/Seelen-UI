import { invoke, SeelenCommand } from "@seelen-ui/lib";
import type { Resource, ResourceId, ResourceKind, ResourceMetadata, Wallpaper } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { ResourceText, ResourceTextAsMarkdown } from "libs/ui/react/components/ResourceText/index.tsx";
import { cx } from "libs/ui/react/utils/styling.ts";
import { convertFileSrc } from "@tauri-apps/api/core";
import { Button, Popconfirm, Tooltip } from "antd";
import type { ComponentChildren } from "preact";
import { useEffect, useState } from "preact/hooks";
import { useTranslation } from "react-i18next";

import { EnvConfig } from "../shared/config/infra.ts";
import cs from "./infra.module.css";
import type { IconName } from "libs/ui/icons.ts";
import { $corruptedWallpapers } from "../shared/signals.ts";

type AnyResource = {
  id: ResourceId;
  metadata: ResourceMetadata;
};

interface ResourceCardProps {
  kind: ResourceKind;
  resource: AnyResource;
  actions?: React.ReactNode;
}

export function ResourceCard({ resource, kind, actions }: ResourceCardProps) {
  const [hasUpdate, setHasUpdate] = useState(false);
  const isCorrupted = kind === "Wallpaper" && $corruptedWallpapers.value.has(resource.id);

  const { t } = useTranslation();

  useEffect(() => {
    async function checkUpdate() {
      if (resource.id.startsWith("@")) {
        return;
      }

      const res = await fetch(`https://product.seelen.io/resource/${resource.id}`);
      const remoteResource: Resource = await res.json();
      const lastUpdateRelease = new Date(remoteResource.updatedAt);
      const writtenAt = new Date(resource.metadata.writtenAt);
      setHasUpdate(lastUpdateRelease > writtenAt);
    }

    checkUpdate();
  }, []);

  const [major = 0, minor = 0, patch = 0] = EnvConfig.version.split(".").map(Number);
  const [majorTarget = 0, minorTarget = 0, patchTarget = 0] = resource.metadata.appTargetVersion || [];

  const targetIsOlder = !!resource.metadata.appTargetVersion &&
    (majorTarget < major ||
      (majorTarget === major && minorTarget < minor) ||
      (majorTarget === major && minorTarget === minor && patchTarget < patch));

  const targetIsNewer = !!resource.metadata.appTargetVersion &&
    (majorTarget > major ||
      (majorTarget === major && minorTarget > minor) ||
      (majorTarget === major && minorTarget === minor && patchTarget > patch));

  return (
    <div
      className={cx(cs.card, {
        [cs.warn!]: targetIsOlder,
        [cs.danger!]: targetIsNewer || isCorrupted,
      })}
    >
      <ResourcePortrait resource={resource} kind={kind}>
        {targetIsOlder && (
          <Tooltip title={t("resources.outdated")}>
            <Icon iconName="IoWarning" className={cs.warning} />
          </Tooltip>
        )}
        {targetIsNewer && (
          <Tooltip title={t("resources.app_outdated")}>
            <Icon iconName="IoWarning" className={cs.danger} />
          </Tooltip>
        )}
        {isCorrupted && (
          <Tooltip title={t("resources.corrupted_wallpaper")}>
            <Icon iconName="MdErrorOutline" className={cs.corrupted} />
          </Tooltip>
        )}
      </ResourcePortrait>

      <div className={cs.header}>
        <ResourceText className={cs.title} text={resource.metadata.displayName} />

        <div className={cs.actionsTop}>
          {!resource.id.startsWith("@") && (
            <Tooltip title={t("resources.see_on_website")}>
              <Button
                type="link"
                href={`https://seelen.io/resources/${compressUuid(resource.id)}`}
                target="_blank"
              >
                <Icon iconName="TbWorldShare" />
              </Button>
            </Tooltip>
          )}

          {hasUpdate && (
            <Tooltip title={t("resources.has_update")} placement="left">
              <Button
                type="link"
                href={`https://seelen.io/resources/${compressUuid(resource.id)}?update`}
                target="_blank"
              >
                <Icon iconName="MdUpdate" />
              </Button>
            </Tooltip>
          )}

          {actions}
        </div>
      </div>

      <div className={cs.body}>
        <ResourceTextAsMarkdown text={resource.metadata.description} />
      </div>

      <div className={cs.footer}>
        {!resource.metadata.bundled && resource.metadata.path.includes("com.seelen.seelen-ui") && (
          <Tooltip title={t("resources.delete")} placement="left">
            <Popconfirm
              title={t("action.confirm")}
              description={t("action.confirm_body")}
              okText={t("yes")}
              cancelText={t("no")}
              onConfirm={() => {
                invoke(SeelenCommand.RemoveResource, { kind, id: resource.id });
              }}
            >
              <Button type="text" danger>
                <Icon iconName="BiTrash" />
              </Button>
            </Popconfirm>
          </Tooltip>
        )}
      </div>
    </div>
  );
}

const icons: Record<ResourceKind, IconName> = {
  Theme: "IoColorPaletteOutline",
  IconPack: "LiaIconsSolid",
  Plugin: "BsPlugin",
  Widget: "BiSolidWidget",
  Wallpaper: "LuWallpaper",
  SoundPack: "PiWaveformBold",
};

interface ResourcePortraitProps {
  resource: AnyResource;
  kind: ResourceKind;
  children?: ComponentChildren;
}

export function ResourceIcon({ kind }: { kind: ResourceKind }) {
  return <Icon className={cs.kindIcon} iconName={icons[kind]!} />;
}

function ResourcePortraitInner({ resource, kind }: ResourcePortraitProps) {
  if (resource.metadata.portrait) {
    return <img src={resource.metadata.portrait} />;
  }

  if (kind === "Wallpaper") {
    const wallpaper = resource as Wallpaper;
    if (wallpaper.thumbnailFilename) {
      return (
        <img
          src={convertFileSrc(`${resource.metadata.path}\\${wallpaper.thumbnailFilename}`)}
          style={{ filter: "blur(0.4px)" }}
          loading="lazy"
        />
      );
    }
  }

  return <ResourceIcon kind={kind} />;
}

export function ResourcePortrait({ resource, kind, children }: ResourcePortraitProps) {
  return (
    <figure className={cs.portrait}>
      <ResourcePortraitInner resource={resource} kind={kind} />
      {children}
    </figure>
  );
}

export function compressUuid(uuid: string): string {
  let hex = uuid.replace(/-/g, "");
  let data = String.fromCharCode.apply(
    null,
    hex.match(/\w{2}/g)!.map(function (a) {
      return parseInt(a, 16);
    }),
  );
  return btoa(data).replace(/\+/g, "-").replace(/\//g, "_").replace(/=/g, "");
}
