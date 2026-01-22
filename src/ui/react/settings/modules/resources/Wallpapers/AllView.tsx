import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { ResourceKind, type Wallpaper } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { path } from "@tauri-apps/api";
import { Button } from "antd";
import { useTranslation } from "react-i18next";
import { NavLink } from "react-router";

import cs from "../infra.module.css";

import { wallpapers } from "../../../state/resources.ts";

import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";
import { ResourceCard } from "../ResourceCard.tsx";

export function AllWallpapersView() {
  const { t } = useTranslation();

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("resources.open_folder")}</b>
          <Button
            type="default"
            onClick={async () => {
              const dataDir = await path.appDataDir();
              invoke(SeelenCommand.OpenFile, {
                path: await path.join(dataDir, "wallpapers"),
              });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <b>{t("resources.import_wallpapers")}</b>
          <Button
            type="default"
            onClick={() => {
              invoke(SeelenCommand.StateRequestWallpaperAddition);
            }}
          >
            <Icon iconName="MdLibraryAdd" />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>{t("resources.discover")}:</span>
          <Button
            href="https://seelen.io/resources/s?category=Wallpaper"
            target="_blank"
            type="link"
          >
            https://seelen.io/resources/s?category=Wallpaper
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <div className={cs.list}>
        {wallpapers.value.map((resource) => <WallpaperItem key={resource.id} resource={resource} />)}
      </div>
    </>
  );
}

function WallpaperItem({ resource }: { resource: Wallpaper }) {
  return (
    <ResourceCard
      key={resource.id}
      resource={resource}
      kind={ResourceKind.Wallpaper}
      actions={
        <>
          <NavLink to={`/wallpaper?${new URLSearchParams({ id: resource.id })}`}>
            <Button type="text">
              <Icon iconName="RiSettings4Fill" />
            </Button>
          </NavLink>
        </>
      }
    />
  );
}
