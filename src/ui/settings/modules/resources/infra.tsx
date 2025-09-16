import { ResourceKind } from "@seelen-ui/lib/types";
import { useTranslation } from "react-i18next";
import { NavLink, Route, Routes } from "react-router";

import cs from "./infra.module.css";

import { RoutePath } from "../../components/navigation/routes";
import { IconPacksView } from "./IconPacks";
import { PluginsView } from "./Plugins";
import { ResourceIcon } from "./ResourceCard";
import { SoundPacksView } from "./SoundPacks";
import { ThemesView } from "./Theme/AllView";
import { AllWallpapersView } from "./Wallpapers/AllView";
import { WidgetsView } from "./Widget/AllView";

const kinds: ResourceKind[] = [
  "Theme",
  "Plugin",
  "Widget",
  "IconPack",
  "Wallpaper",
  "SoundPack",
];

export function ResourcesView() {
  return (
    <Routes>
      <Route index Component={KindSelector} />
      <Route path="theme" Component={ThemesView} />
      <Route path="plugin" Component={PluginsView} />
      <Route path="widget" Component={WidgetsView} />
      <Route path="wallpaper" Component={AllWallpapersView} />
      <Route path="iconpack" Component={IconPacksView} />
      <Route path="soundpack" Component={SoundPacksView} />
    </Routes>
  );
}

function KindSelector() {
  const { t } = useTranslation();

  return (
    <div className={cs.kindSelector}>
      {kinds.map((kind) => (
        <NavLink
          key={kind}
          to={`${RoutePath.Resource}/${kind.toLowerCase()}`}
          className={cs.kind}
        >
          <ResourceIcon kind={kind} />
          <b>{t(`header.labels.${kind.toLowerCase()}`)}</b>
        </NavLink>
      ))}
    </div>
  );
}
