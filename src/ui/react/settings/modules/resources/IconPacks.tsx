import { type IconPack, type IconPackId, ResourceKind } from "@seelen-ui/lib/types";
import { Switch } from "antd";
import { Reorder } from "framer-motion";
import { useTranslation } from "react-i18next";
import { useState } from "preact/hooks";

import cs from "./infra.module.css";

import { setActiveIconPacks, settings } from "../../state/mod.ts";
import { iconPacks as allIconPacks } from "../../state/resources.ts";

import { resolveDisplayName, ResourceCard, ResourceListHeader } from "./ResourceCard.tsx";

export function IconPacksView() {
  const activeIds = settings.value.activeIconPacks;
  const { t, i18n } = useTranslation();
  const [search, setSearch] = useState("");

  function toggleIconPack(id: IconPackId) {
    if (activeIds.includes(id)) {
      setActiveIconPacks(activeIds.filter((x) => x !== id));
    } else {
      setActiveIconPacks([...activeIds, id]);
    }
  }

  function onReorder(activeIconPacks: IconPackId[]) {
    setActiveIconPacks(activeIconPacks);
  }

  const allFiltered = search
    ? allIconPacks.value.filter((pack) =>
      resolveDisplayName(pack.metadata.displayName, i18n.language)
        .toLowerCase()
        .includes(search.toLowerCase())
    )
    : allIconPacks.value;

  const disabled: IconPack[] = [];
  const enabled: IconPack[] = [];
  for (const pack of allFiltered) {
    if (activeIds.includes(pack.id)) {
      enabled.push(pack);
    } else {
      disabled.push(pack);
    }
  }
  enabled.sort((a, b) => activeIds.indexOf(a.id) - activeIds.indexOf(b.id));

  return (
    <div className={cs.list}>
      <ResourceListHeader
        discoverUrl="https://seelen.io/resources/s?category=IconPack"
        search={search}
        onSearch={setSearch}
      />

      <b>{t("general.icon_pack.selected")}</b>
      <Reorder.Group values={activeIds} onReorder={onReorder} className={cs.reorderGroup}>
        {enabled.map((iconPack) => (
          <Reorder.Item key={iconPack.id} value={iconPack.id}>
            <ResourceCard
              resource={iconPack}
              kind={ResourceKind.IconPack}
              actions={iconPack.id === "@system/icon-pack"
                ? undefined
                : <Switch defaultChecked onChange={() => toggleIconPack(iconPack.id)} />}
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
          actions={<Switch defaultChecked={false} onChange={() => toggleIconPack(iconPack.id)} />}
        />
      ))}
    </div>
  );
}
