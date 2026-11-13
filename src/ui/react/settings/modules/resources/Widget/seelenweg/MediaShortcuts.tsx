import type { WegMediaShortcutProfile } from "@seelen-ui/lib/types";
import { Input, Switch } from "antd";
import { useTranslation } from "react-i18next";
import type { JSX } from "preact";
import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../../components/SettingsBox/index.tsx";
import { isMediaShortcut, mediaShortcutProfile } from "src/ui/svelte/weg-preview/media.ts";
import { getWegConfig, patchWegConfig } from "./application.ts";

const emptyProfile: WegMediaShortcutProfile = { enabled: false, previous: "", togglePlayPause: "", next: "" };

export function MediaShortcuts(): JSX.Element {
  const { t } = useTranslation();
  const profiles = getWegConfig().mediaShortcuts;

  function update(exe: string, patch: Partial<WegMediaShortcutProfile>): void {
    const current = getWegConfig().mediaShortcuts;
    patchWegConfig({
      mediaShortcuts: { ...current, [exe]: { ...emptyProfile, ...mediaShortcutProfile(current, exe), ...patch } },
    });
  }

  return (
    <SettingsGroup>
      <SettingsSubGroup label={t("weg.media_shortcuts.label")}>
        <p>{t("weg.media_shortcuts.description")}</p>
        {[{ exe: "cloudmusic.exe", key: "netease" }, { exe: "qqmusic.exe", key: "qq_music" }].map(({ exe, key }) => {
          const profile = mediaShortcutProfile(profiles, exe) ?? emptyProfile;
          return (
            <SettingsSubGroup key={exe} label={t(`weg.media_shortcuts.${key}`)}>
              <SettingsOption label={t("weg.media_shortcuts.enabled")}>
                <Switch checked={profile.enabled} onChange={(enabled) => update(exe, { enabled })} />
              </SettingsOption>
              {(["previous", "togglePlayPause", "next"] as const).map((action) => (
                <SettingsOption key={action} label={t(`weg.media_shortcuts.${action}`)}>
                  <Input
                    value={profile[action]}
                    aria-label={t(`weg.media_shortcuts.${action}`)}
                    placeholder="Ctrl+Alt+F12"
                    status={profile[action] && !isMediaShortcut(profile[action]) ? "error" : undefined}
                    onChange={(e) =>
                      update(exe, { [action]: e.currentTarget.value })}
                  />
                </SettingsOption>
              ))}
            </SettingsSubGroup>
          );
        })}
        <p>{t("weg.media_shortcuts.state_unknown")}</p>
      </SettingsSubGroup>
    </SettingsGroup>
  );
}
