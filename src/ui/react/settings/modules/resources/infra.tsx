import { ResourceKind } from "@seelen-ui/lib/types";
import { useTranslation } from "react-i18next";
import { NavLink, Route, Routes } from "react-router";

import cs from "./infra.module.css";

import { RoutePath } from "../../components/navigation/routes.tsx";
import { IconPacksView } from "./IconPacks.tsx";
import { PluginsView } from "./Plugins.tsx";
import { ResourceIcon } from "./ResourceCard.tsx";
import { SoundPacksView } from "./SoundPacks.tsx";
import { ThemesView } from "./Theme/AllView.tsx";
import { AllWallpapersView } from "./Wallpapers/AllView.tsx";
import { WidgetsView } from "./Widget/AllView.tsx";

const kinds = Object.values(ResourceKind);
const DISABLED_KINDS: ResourceKind[] = [ResourceKind.SoundPack];

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
      {kinds.map((kind) => {
        const disabled = DISABLED_KINDS.includes(kind);
        return disabled
          ? (
            <div key={kind} className={`${cs.kind} ${cs.kindDisabled}`}>
              <ResourceIcon kind={kind} />
              <b>{t(`header.labels.${kind.toLowerCase()}`)}</b>
            </div>
          )
          : (
            <NavLink
              key={kind}
              to={`${RoutePath.Resource}/${kind.toLowerCase()}`}
              className={cs.kind}
            >
              <ResourceIcon kind={kind} />
              <b>{t(`header.labels.${kind.toLowerCase()}`)}</b>
            </NavLink>
          );
      })}
    </div>
  );
}
