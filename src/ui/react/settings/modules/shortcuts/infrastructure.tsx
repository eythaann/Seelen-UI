import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import type { SluHotkey } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Input, Switch, Tooltip } from "antd";
import { useTranslation } from "react-i18next";
import { useEffect } from "react";

import {
  getHotkeysGroups,
  getShortcutsConfig,
  isWidgetEnabled,
  resetShortcuts,
  setShortcutsEnabled,
  shortcutsError,
  updateShortcut,
  validateShortcuts,
} from "./application.ts";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";

export function Shortcuts() {
  const shortcutsConfig = getShortcutsConfig();
  const { enabled, appCommands } = shortcutsConfig;

  const { t } = useTranslation();

  useEffect(() => {
    validateShortcuts(appCommands);
  }, [appCommands]);

  function onToogleShortcuts(enabled: boolean) {
    setShortcutsEnabled(enabled);
  }

  function onShortcutChanged(id: string, keys: string[]) {
    updateShortcut(id, keys);
  }

  function onReset() {
    resetShortcuts();
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

      <SettingsGroup>{groups.virtualDesktop.main.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.virtualDesktop.switch.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.virtualDesktop.move.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.virtualDesktop.send.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.windowManager.state.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.windowManager.sizing.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.windowManager.positioning.map(mapHokey)}</SettingsGroup>

      <SettingsGroup>{groups.windowManager.tilingFocus.map(mapHokey)}</SettingsGroup>

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

function Shortcut({
  hotkey: { id, action, keys, readonly, system, attached_to },
  onChanged,
}: ShortcutProps) {
  const { t } = useTranslation();

  const isEnabled = !attached_to || isWidgetEnabled(attached_to);

  const args: Record<string, number | string> = "index" in action ? { 0: action.index } : {};
  const hasError = shortcutsError.value.has(id);

  function onEdit() {
    if (readonly || system || !isEnabled) {
      return;
    }

    invoke(SeelenCommand.RequestToUserInputShortcut, {
      callbackEvent: "finished",
    });

    Widget.getCurrent().webview.once<null | string[]>("finished", (e) => {
      // Cancel if user didn't input at least 2 keys
      if (e.payload && e.payload.length >= 2) {
        onChanged(e.payload);
      }
    });
  }

  let tooltipTitle: string | undefined;
  if (readonly || system) {
    tooltipTitle = t("shortcuts.readonly_tooltip");
  } else if (hasError) {
    tooltipTitle = t("shortcuts.duplicate_error");
  }

  return (
    <SettingsOption
      disabled={!isEnabled}
      label={t(`shortcuts.labels.${action.name}`, args)}
      action={
        <Tooltip title={tooltipTitle} placement="left">
          <Input
            value={keys.join(" + ")}
            onClick={onEdit}
            status={hasError ? "error" : undefined}
            readOnly
            disabled={!isEnabled}
          />
        </Tooltip>
      }
    />
  );
}
