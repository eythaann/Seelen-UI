import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { SluHotkey } from "@seelen-ui/lib/types";
import { Icon } from "@shared/components/Icon";
import { Button, Input, Switch, Tooltip } from "antd";
import Compact from "antd/es/space/Compact";
import { cloneDeep } from "lodash";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";

import { defaultSettings } from "../shared/store/app/default.ts";
import { newSelectors, RootActions } from "../shared/store/app/reducer.ts";
import { getHotkeysGroups } from "./application.tsx";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";

export function Shortcuts() {
  const { enabled, appCommands } = useSelector(newSelectors.shortcuts);

  const d = useDispatch();
  const { t } = useTranslation();

  function onToogleShortcuts(enabled: boolean) {
    d(RootActions.setShortcuts({ enabled, appCommands }));
  }

  function onShortcutChanged(id: string, keys: string[]) {
    d(
      RootActions.setShortcuts({
        enabled,
        appCommands: appCommands.map((c) => (c.id === id ? { ...c, keys } : c)),
      }),
    );
  }

  function onReset() {
    d(
      RootActions.setShortcuts({
        enabled,
        appCommands: cloneDeep(defaultSettings.shortcuts.appCommands),
      }),
    );
  }

  const groups = getHotkeysGroups(appCommands);

  function mapHokey(hotkey: SluHotkey) {
    return (
      <Shortcut
        key={hotkey.id}
        hotkey={hotkey}
        onChanged={(keys) => onShortcutChanged(hotkey.id, keys)}
      />
    );
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption
          label={t("shortcuts.enable")}
          tip={t("shortcuts.enable_tooltip")}
          action={<Switch value={enabled} onChange={onToogleShortcuts} />}
        />

        <SettingsOption
          label={t("shortcuts.reset")}
          action={
            <Button onClick={onReset}>
              <Icon iconName="RiResetLeftLine" />
            </Button>
          }
        />
      </SettingsGroup>

      <SettingsGroup>{groups.launcher.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.virtualDesktop.main.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>
        {groups.virtualDesktop.switch.map(mapHokey)}
      </SettingsGroup>

      <SettingsGroup>{groups.virtualDesktop.move.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.virtualDesktop.send.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.windowManager.state.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.windowManager.sizing.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>
        {groups.windowManager.positioning.map(mapHokey)}
      </SettingsGroup>

      <SettingsGroup>
        {groups.windowManager.tilingFocus.map(mapHokey)}
      </SettingsGroup>

      {/* TODO implement live layout modification */}
      {/* <SettingsGroup>{groups.windowManager.tilingLayout.map(mapHokey)}</SettingsGroup> */}

      <SettingsGroup>{groups.weg.map(mapHokey)}</SettingsGroup>

      {/* TODO implement wallpaper change shortcut */}
      {/* <SettingsGroup>{groups.wallpaperManager.map(mapHokey)}</SettingsGroup> */}

      <SettingsGroup>{groups.misc.map(mapHokey)}</SettingsGroup>
    </>
  );
}

interface ShortcutProps {
  hotkey: SluHotkey;
  onChanged: (keys: string[]) => void;
}

function Shortcut({ hotkey: { action, keys }, onChanged }: ShortcutProps) {
  const { t } = useTranslation();

  const args: Record<string, number | string> = "index" in action ? { 0: action.index } : {};

  function onEdit() {
    invoke(SeelenCommand.RequestToUserInputShortcut, {
      callbackEvent: "finished",
    });
    Widget.getCurrent().webview.once<null | string[]>("finished", (e) => {
      if (e.payload) {
        onChanged(e.payload);
      }
    });
  }

  return (
    <SettingsOption
      label={t(`shortcuts.labels.${action.name}`, args)}
      action={
        <Compact>
          <Input value={keys.join(" + ")} readOnly />
          <Tooltip title={t("shortcuts.readonly_tooltip")}>
            <Button onClick={onEdit} style={{ minWidth: 32 }}>
              <Icon iconName="AiOutlineEdit" />
            </Button>
          </Tooltip>
        </Compact>
      }
    />
  );
}
