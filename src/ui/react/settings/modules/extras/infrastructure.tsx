import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { process } from "@seelen-ui/lib/tauri";
import { isDev, wasInstalledUsingMSIX } from "@shared";
import { Icon } from "@shared/components/Icon";
import { Button, Select, Switch, Tooltip } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";

import { EnvConfig } from "../shared/config/infra.ts";
import cs from "./infra.module.css";

import { newSelectors, RootActions } from "../shared/store/app/reducer.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";
import { UpdateChannel } from "@seelen-ui/lib/types";

export function Information() {
  const [isMsixBuild, setIsMsixBuild] = useState(true);
  const [isDevMode, setIsDevMode] = useState(false);

  const drpc = useSelector(newSelectors.drpc);
  const updaterSettings = useSelector(newSelectors.updater);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useEffect(() => {
    wasInstalledUsingMSIX().then(setIsMsixBuild);
    isDev().then(setIsDevMode);
  }, []);

  function onToggleDrpc(value: boolean) {
    dispatch(RootActions.setDrpc(value));
  }

  function onChangeUpdateChannel(channel: UpdateChannel) {
    dispatch(RootActions.setUpdater({ ...updaterSettings, channel }));
  }

  return (
    <div className={cs.info}>
      <figure className={cs.logo}>
        <img src="./company_logo.svg" alt="Seelen Corp." />
        <figcaption>Seelen Corp.</figcaption>
      </figure>

      <SettingsGroup>
        <SettingsSubGroup label="Seelen UI">
          <SettingsOption>
            <span>{t("extras.version")}:</span>
            <span className={cs.version}>
              v{EnvConfig.version} {isDevMode && "(dev)"} {isMsixBuild && "(msix)"}
            </span>
          </SettingsOption>
          <SettingsOption>
            <span>{t("update.channel")}</span>
            <Select
              value={updaterSettings.channel}
              disabled={isMsixBuild}
              onChange={onChangeUpdateChannel}
              options={Object.values(UpdateChannel).map((c) => ({
                value: c,
                label: c,
              }))}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("extras.links")}>
          <SettingsOption>
            <span>Github:</span>
            <a href="https://github.com/eythaann/seelen-ui" target="_blank">
              github.com/eythaann/seelen-ui
            </a>
          </SettingsOption>
          <SettingsOption>
            <span>Discord:</span>
            <a href="https://discord.gg/ABfASx5ZAJ" target="_blank">
              discord.gg/ABfASx5ZAJ
            </a>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>Discord RPC</b>
          <Switch value={drpc} onChange={onToggleDrpc} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b style={{ display: "flex", alignItems: "center", gap: "4px" }}>
            {t("extras.clear_icons")}
            <Tooltip title={t("extras.clear_icons_tooltip")}>
              <Icon iconName="LuCircleHelp" />
            </Tooltip>
          </b>
          <Button
            type="dashed"
            danger
            onClick={() => invoke(SeelenCommand.StateDeleteCachedIcons)}
            style={{ width: "50px" }}
          >
            <Icon iconName="IoReload" size={12} />
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t("extras.relaunch")}</b>
          <Button type="dashed" onClick={() => process.relaunch()} style={{ width: "50px" }}>
            <Icon iconName="IoReload" size={12} />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <b>{t("extras.exit")}</b>
          <Button type="dashed" danger onClick={() => process.exit(0)} style={{ width: "50px" }}>
            <Icon iconName="IoClose" />
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
