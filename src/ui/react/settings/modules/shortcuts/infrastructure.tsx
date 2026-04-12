import { invoke, SeelenCommand, Widget } from "@seelen-ui/lib";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Input, Switch, Tooltip } from "antd";
import { useTranslation } from "react-i18next";
import { useEffect } from "react";

import {
  getShortcutGroups,
  getShortcutsConfig,
  resetShortcuts,
  setShortcutsEnabled,
  type ShortcutEntry,
  shortcutsError,
  updateShortcut,
  validateShortcuts,
} from "./application.ts";

import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { isWidgetEnabled } from "../resources/Widget/application.ts";
import Compact from "antd/es/space/Compact";

export function Shortcuts() {
  const { enabled } = getShortcutsConfig();
  const groups = getShortcutGroups();

  const { t } = useTranslation();

  const allEntries = [
    ...Array.from(groups.byWidget.values()).flatMap((g) => g.entries),
    ...Object.values(groups.system).flat(),
  ];

  useEffect(() => {
    validateShortcuts(allEntries);
  }, [JSON.stringify(allEntries.map((e) => e.keys))]);

  function mapEntry(entry: ShortcutEntry) {
    return <Shortcut key={entry.id} entry={entry} onChanged={(keys) => updateShortcut(entry, keys)} />;
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption
          label={t("shortcuts.enable")}
          tip={t("shortcuts.enable_tooltip")}
          action={<Switch value={enabled} onChange={setShortcutsEnabled} />}
        />
        <SettingsOption
          label={t("shortcuts.reset")}
          action={
            <Button onClick={resetShortcuts}>
              <Icon iconName="RiResetLeftLine" />
            </Button>
          }
        />
      </SettingsGroup>

      {/* Virtual Desktop */}
      <SettingsGroup>
        <SettingsSubGroup label={t("header.labels.virtual_desk")}>
          {groups.system.vdMain.map(mapEntry)}
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("header.labels.virtual_desk")}>
          {groups.system.vdSwitch.map(mapEntry)}
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("header.labels.virtual_desk")}>
          {groups.system.vdMove.map(mapEntry)}
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label={t("header.labels.virtual_desk")}>
          {groups.system.vdSend.map(mapEntry)}
        </SettingsSubGroup>
      </SettingsGroup>

      {/* Widget groups */}
      {Array.from(groups.byWidget.values()).map(({ widget, entries }) => (
        <SettingsGroup key={widget.id}>
          <SettingsSubGroup label={<ResourceText text={widget.metadata.displayName} />}>
            {entries.map(mapEntry)}
          </SettingsSubGroup>
        </SettingsGroup>
      ))}

      {/* Misc */}
      <SettingsGroup>{groups.system.misc.map(mapEntry)}</SettingsGroup>
    </>
  );
}

interface ShortcutProps {
  entry: ShortcutEntry;
  onChanged: (keys: string[]) => void;
}

function Shortcut({ entry, onChanged }: ShortcutProps) {
  const { id, label, keys, readonly } = entry;

  const isAttachedWidgetEnabled = entry.widgetId ? isWidgetEnabled(entry.widgetId) : true;

  const { t } = useTranslation();
  const hasError = shortcutsError.value.has(id);

  function onEdit() {
    if (readonly) return;

    invoke(SeelenCommand.RequestToUserInputShortcut, { callbackEvent: "finished" });
    Widget.getCurrent().webview.once<null | string[]>("finished", (e) => {
      if (e.payload && e.payload.length >= 2) {
        onChanged(e.payload);
      }
    });
  }

  let inputTooltip: string | undefined = undefined;
  if (hasError) {
    inputTooltip = t("shortcuts.duplicate_error");
  } else if (!isAttachedWidgetEnabled) {
    inputTooltip = t("shortcuts.disabled_tooltip");
  }

  return (
    <SettingsOption
      label={<ResourceText text={label} />}
      action={
        <Compact>
          <Tooltip title={inputTooltip} placement="left">
            <Input
              value={keys.join(" + ")}
              status={hasError ? "error" : undefined}
              readOnly
              disabled={!isAttachedWidgetEnabled}
            />
          </Tooltip>

          <Tooltip title={readonly ? t("shortcuts.readonly_tooltip") : undefined}>
            <Button type="primary" disabled={readonly} onClick={onEdit}>
              <Icon iconName="IoPencilOutline" />
            </Button>
          </Tooltip>
        </Compact>
      }
    />
  );
}
