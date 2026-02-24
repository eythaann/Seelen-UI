import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { ResourceKind, type Wallpaper } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button } from "antd";
import { useTranslation } from "react-i18next";
import { useState } from "preact/hooks";
import { NavLink } from "react-router";

import cs from "../infra.module.css";

import { wallpapers } from "../../../state/resources.ts";

import { resolveDisplayName, ResourceCard, ResourceListHeader } from "../ResourceCard.tsx";

export function AllWallpapersView() {
  const { t, i18n } = useTranslation();
  const [search, setSearch] = useState("");

  const filtered = search
    ? wallpapers.value.filter((w) =>
      resolveDisplayName(w.metadata.displayName, i18n.language).toLowerCase().includes(search.toLowerCase())
    )
    : wallpapers.value;

  return (
    <>
      <ResourceListHeader
        discoverUrl="https://seelen.io/resources/s?category=Wallpaper"
        search={search}
        onSearch={setSearch}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <b>{t("resources.import_wallpapers")}</b>
          <Button
            type="default"
            onClick={() => {
              invoke(SeelenCommand.StateRequestWallpaperAddition);
            }}
          >
            <Icon iconName="MdLibraryAdd" />
          </Button>
        </div>
      </ResourceListHeader>

      <div className={cs.list}>
        {filtered.map((resource) => <WallpaperItem key={resource.id} resource={resource} />)}
      </div>
    </>
  );
}

function WallpaperItem({ resource }: { resource: Wallpaper }) {
  return (
    <ResourceCard
      resource={resource}
      kind={ResourceKind.Wallpaper}
      actions={
        <NavLink to={`/wallpaper?${new URLSearchParams({ id: resource.id })}`}>
          <Button type="text">
            <Icon iconName="RiSettings4Fill" />
          </Button>
        </NavLink>
      }
    />
  );
}
