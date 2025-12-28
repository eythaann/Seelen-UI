import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { type IconPack, type IconPackId, ResourceKind } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { path } from "@tauri-apps/api";
import { Button, Switch } from "antd";
import { Reorder } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";

import cs from "./infra.module.css";

import { RootActions } from "../shared/store/app/reducer.ts";
import { RootSelectors } from "../shared/store/app/selectors.ts";

import { SettingsGroup, SettingsOption } from "../../components/SettingsBox/index.tsx";
import { ResourceCard } from "./ResourceCard.tsx";

export function IconPacksView() {
  const activeIds = useSelector(RootSelectors.activeIconPacks);
  const allIconPacks = useSelector(RootSelectors.availableIconPacks);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  function toggleIconPack(id: IconPackId) {
    if (activeIds.includes(id)) {
      dispatch(
        RootActions.setActiveIconPacks(activeIds.filter((x) => x !== id)),
      );
    } else {
      dispatch(RootActions.setActiveIconPacks([...activeIds, id]));
    }
  }

  function onReorder(activeIconPacks: IconPackId[]) {
    dispatch(RootActions.setActiveIconPacks(activeIconPacks));
  }

  const disabled: IconPack[] = [];
  const enabled: IconPack[] = [];
  for (const pack of allIconPacks) {
    if (activeIds.includes(pack.id)) {
      enabled.push(pack);
    } else {
      disabled.push(pack);
    }
  }
  enabled.sort((a, b) => activeIds.indexOf(a.id) - activeIds.indexOf(b.id));

  return (
    <div className={cs.list}>
      <SettingsGroup>
        <SettingsOption>
          <b>{t("resources.open_folder")}</b>
          <Button
            type="default"
            onClick={async () => {
              const dataDir = await path.appDataDir();
              invoke(SeelenCommand.OpenFile, {
                path: await path.join(dataDir, "iconpacks"),
              });
            }}
          >
            <Icon iconName="PiFoldersDuotone" />
          </Button>
        </SettingsOption>
      </SettingsGroup>

      <b>{t("general.icon_pack.selected")}</b>
      <Reorder.Group
        values={activeIds}
        onReorder={onReorder}
        className={cs.reorderGroup}
      >
        {enabled.map((iconPack) => (
          <Reorder.Item key={iconPack.id} value={iconPack.id}>
            <ResourceCard
              resource={iconPack}
              kind={ResourceKind.IconPack}
              actions={iconPack.id === "@system/icon-pack" ? undefined : (
                <Switch
                  defaultChecked
                  onChange={() => toggleIconPack(iconPack.id)}
                />
              )}
            />
          </Reorder.Item>
        ))}
      </Reorder.Group>

      <b>{t("general.icon_pack.available")}</b>
      {disabled.map((iconPack) => (
        <ResourceCard
          key={iconPack.id}
          resource={iconPack}
          kind={ResourceKind.IconPack}
          actions={
            <Switch
              defaultChecked={false}
              onChange={() => toggleIconPack(iconPack.id)}
            />
          }
        />
      ))}
    </div>
  );
}
