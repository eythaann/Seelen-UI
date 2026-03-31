import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { process } from "@seelen-ui/lib/tauri";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Select, Switch, Tooltip } from "antd";
import { useTranslation } from "react-i18next";

import { EnvConfig } from "../shared/config/infra.ts";
import cs from "./infra.module.css";

import {
  getDrpc,
  getStreamingMode,
  getUpdaterSettings,
  patchUpdaterSettings,
  setDrpc,
  setStreamingMode,
} from "./application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";
import { UpdateChannel } from "@seelen-ui/lib/types";
import { SessionView } from "./session/infra.tsx";

const [isDevMode, isMsixBuild, isFixed] = await Promise.all([
  invoke(SeelenCommand.IsDevMode),
  invoke(SeelenCommand.IsAppxPackage),
  invoke(SeelenCommand.HasFixedRuntime),
]);

export function Information() {
  const drpc = getDrpc();
  const streamingMode = getStreamingMode();
  const updaterSettings = getUpdaterSettings();

  const { t } = useTranslation();

  function onToggleDrpc(value: boolean) {
    setDrpc(value);
  }

  function onToggleStreamingMode(value: boolean) {
    setStreamingMode(value);
  }

  function onChangeUpdateChannel(channel: UpdateChannel) {
    patchUpdaterSettings({ channel });
  }

  return (
    <div className={cs.info}>
      <div className={cs.sessionContainer}>
        <SessionView />
      </div>

      <SettingsGroup>
        <SettingsSubGroup label="Seelen UI">
          <SettingsOption
            label={t("extras.version")}
            description={isFixed ? t("extras.version_fixed") : false}
            action={
              <span className={cs.version}>
                v{EnvConfig.version} {isDevMode && "(dev)"} {isMsixBuild && "(msix)"} {isFixed && "(fixed)"}
              </span>
            }
          />
          <SettingsOption
            label={t("update.channel")}
            action={
              <Select
                value={updaterSettings.channel}
                disabled={isMsixBuild}
                onChange={onChangeUpdateChannel}
                options={Object.values(UpdateChannel).map((c) => ({
                  value: c,
                  label: c,
                }))}
              />
            }
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>Discord RPC</b>
          <Switch value={drpc} onChange={onToggleDrpc} />
        </SettingsOption>
        <SettingsOption
          label={t("extras.streaming_mode")}
          description={t("extras.streaming_mode_description")}
          action={<Switch value={streamingMode} onChange={onToggleStreamingMode} />}
        />
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
