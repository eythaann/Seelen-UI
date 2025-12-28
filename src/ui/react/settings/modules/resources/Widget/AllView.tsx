import { SeelenCommand } from "@seelen-ui/lib";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { path } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "antd";
import { useTranslation } from "react-i18next";
import { useSelector } from "react-redux";
import { NavLink } from "react-router";

import cs from "../infra.module.css";

import { RootSelectors } from "../../shared/store/app/selectors.ts";

import { SettingsGroup, SettingsOption } from "../../../components/SettingsBox/index.tsx";
import { ResourceCard } from "../ResourceCard.tsx";
import { ResourceKind } from "node_modules/@seelen-ui/lib/esm/gen/types/ResourceKind";

export function WidgetsView() {
  const widgets = useSelector(RootSelectors.widgets);

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
                path: await path.join(dataDir, "widgets"),
              });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>{t("resources.discover")}:</span>
          <Button
            href="https://seelen.io/resources/s?category=Widget"
            target="_blank"
            type="link"
          >
            https://seelen.io/resources/s?category=Widget
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <div className={cs.list}>
        {widgets.map((widget) => (
          <ResourceCard
            key={widget.id}
            resource={widget}
            kind={ResourceKind.Widget}
            actions={
              <>
                {!["@seelen/settings", "@seelen/popup"].includes(widget.id) && (
                  <NavLink to={`/widget/${widget.id.replace("@", "")}`}>
                    <Button type="text">
                      <Icon iconName="RiSettings4Fill" />
                    </Button>
                  </NavLink>
                )}
              </>
            }
          />
        ))}
      </div>
    </>
  );
}
