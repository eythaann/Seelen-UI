import { process } from "@seelen-ui/lib/tauri";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Button } from "antd";
import React from "react";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { NavLink, useLocation, useSearchParams } from "react-router";

import { SaveStore } from "../../modules/shared/store/infra.ts";
import { useAppSelector } from "../../modules/shared/utils/infra.ts";

import { RootActions } from "../../modules/shared/store/app/reducer.ts";
import { RootSelectors } from "../../modules/shared/store/app/selectors.ts";

import { RouteExtraInfo } from "./ExtraInfo.tsx";
import { UpdateButton } from "./UpdateButton.tsx";
import cs from "./index.module.css";

export const Header = () => {
  const widgets = useSelector(RootSelectors.widgets);
  const themes = useSelector(RootSelectors.availableThemes);

  const hasChanges = useAppSelector(RootSelectors.toBeSaved);
  const shouldRestart = useAppSelector(RootSelectors.toBeRestarted);

  const location = useLocation();
  const [searchParams] = useSearchParams();
  const dispatch = useDispatch();
  const { t } = useTranslation();

  const cancelChanges = () => {
    dispatch(RootActions.restoreToLastLoaded());
  };

  const SaveOrQuit = async () => {
    if (hasChanges) {
      await SaveStore();
      if (shouldRestart) {
        await process.relaunch();
      }
    } else {
      await getCurrentWebviewWindow().close();
    }
  };

  const saveBtnLabel = shouldRestart ? t("save_and_restart") : t("save");

  let label: React.ReactNode = <span>null!?</span>;
  let parts = location.pathname === "/" ? ["home"] : location.pathname.split("/").filter(Boolean);

  if (parts[0] === "widget") {
    const widgetId = searchParams.get("id");
    const widget = widgets.find((w) => w.id === widgetId);
    label = widget ? <ResourceText text={widget.metadata.displayName} /> : <span>{widgetId}</span>;
  } else if (parts[0] === "theme") {
    const themeId = searchParams.get("id");
    const theme = themes.find((t) => t.id === themeId);
    label = theme ? <ResourceText text={theme.metadata.displayName} /> : <span>{themeId}</span>;
  } else {
    if (parts[0] === "wallpaper") {
      parts = ["resources", "wallpaper", "config"];
    }

    label = parts.map((part, idx) => (
      <React.Fragment key={part}>
        {idx !== parts.length - 1
          ? (
            <NavLink to={`/${parts.slice(0, idx + 1).join("/")}`} className={cs.part}>
              {t(`header.labels.${part}`)}
            </NavLink>
          )
          : <span className={cs.part}>{t(`header.labels.${part}`)}</span>}
        {++idx < parts.length ? ">" : ""}
      </React.Fragment>
    ));
  }

  const ExtraInfo = RouteExtraInfo[location.pathname];

  return (
    <div className={cs.Header} data-tauri-drag-region>
      <div className={cs.title}>
        {label}
        {ExtraInfo && <ExtraInfo />}
      </div>
      <div className={cs.actions}>
        <UpdateButton />
        <Button
          style={{ minWidth: 60 }}
          type="default"
          danger
          disabled={!hasChanges}
          onClick={cancelChanges}
        >
          {t("cancel")}
        </Button>
        <Button style={{ minWidth: 60 }} type="primary" danger={!hasChanges} onClick={SaveOrQuit}>
          {hasChanges ? saveBtnLabel : t("close")}
        </Button>
      </div>
    </div>
  );
};
