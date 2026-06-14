import { ResourceKind } from "@seelen-ui/lib/types";
import { cx } from "libs/ui/react/utils/styling.ts";
import { useTranslation } from "react-i18next";
import { NavLink, Route, Routes } from "react-router";

import cs from "./infra.module.css";

import { RoutePath } from "../../components/navigation/routes.tsx";
import { IconPacksView } from "./IconPacks.tsx";
import { PluginsView } from "./Plugins.tsx";
import { ResourceIcon } from "./ResourceCard.tsx";
import { SoundPacksView } from "./SoundPacks.tsx";
import { ThemesView } from "./Theme/AllView.tsx";
import { ResourceUpdatesModal } from "./UpdatesModal.tsx";
import { AllWallpapersView } from "./Wallpapers/AllView.tsx";
import { WidgetsView } from "./Widget/AllView.tsx";

const kinds = [
  ResourceKind.Theme,
  ResourceKind.IconPack,
  ResourceKind.Wallpaper,
  ResourceKind.Widget,
  ResourceKind.Plugin,
  ResourceKind.SoundPack,
];
const DISABLED_KINDS: ResourceKind[] = [ResourceKind.SoundPack];

export function ResourcesView() {
  return (
    <>
      <ResourceUpdatesModal />
      <Routes>
        <Route index Component={() => <KindSelector />} />
        <Route path="theme" Component={ThemesView} />
        <Route path="plugin" Component={PluginsView} />
        <Route path="widget" Component={WidgetsView} />
        <Route path="wallpaper" Component={AllWallpapersView} />
        <Route path="iconpack" Component={IconPacksView} />
        <Route path="soundpack" Component={SoundPacksView} />
      </Routes>
    </>
  );
}

export function KindSelector({ compact = false }: { compact?: boolean }) {
  const { t } = useTranslation();

  return (
    <div className={cx(cs.kindSelector, { [cs.compact!]: compact })}>
      {kinds.map((kind) => {
        const disabled = DISABLED_KINDS.includes(kind);
        const label = t(`header.labels.${kind.toLowerCase()}`);

        if (disabled) {
          if (compact) return null;
          return (
            <div key={kind} className={cx(cs.kind, cs.kindDisabled)}>
              <ResourceIcon kind={kind} />
              <b>{label}</b>
              <span className={cs.kindSoon}>{t("resources.coming_soon")}</span>
            </div>
          );
        }

        return (
          <NavLink
            key={kind}
            to={`${RoutePath.Resource}/${kind.toLowerCase()}`}
            className={({ isActive }) => cx(cs.kind, { [cs.kindActive!]: isActive })}
          >
            <ResourceIcon kind={kind} />
            <b>{label}</b>
          </NavLink>
        );
      })}
    </div>
  );
}
