import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { path } from "@tauri-apps/api";
import { Button } from "antd";
import { useTranslation } from "react-i18next";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";

export function SoundPacksView() {
  const { t } = useTranslation();

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("resources.open_folder")}</b>
          <Button
            type="default"
            onClick={async () => {
              const dataDir = await path.appDataDir();
              invoke(SeelenCommand.OpenFile, {
                path: await path.join(dataDir, "soundpacks"),
              });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
