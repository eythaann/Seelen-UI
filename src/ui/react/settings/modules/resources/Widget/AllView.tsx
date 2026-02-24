import { ResourceKind, type Widget } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Switch } from "antd";
import { useTranslation } from "react-i18next";
import { useState } from "preact/hooks";
import { NavLink } from "react-router";

import cs from "../infra.module.css";

import { widgets } from "../../../state/resources.ts";
import { getWidgetConfig, patchWidgetConfig } from "./application.ts";

import { resolveDisplayName, ResourceCard, ResourceListHeader } from "../ResourceCard.tsx";

const SYSTEM_WIDGET_IDS = ["@seelen/settings", "@seelen/popup", "@seelen/context-menu"];

function WidgetItem({ widget }: { widget: Widget }) {
  const rootConfig = getWidgetConfig(widget.id);
  const enabled = rootConfig?.enabled ?? (widget.loader !== "Legacy" && !!widget.metadata.bundled);

  const query = new URLSearchParams({ id: widget.id });

  return (
    <ResourceCard
      resource={widget}
      kind={ResourceKind.Widget}
      actions={
        <>
          {widget.settings.length > 0 && (
            <NavLink to={`/widget?${query}`}>
              <Button type="text">
                <Icon iconName="RiSettings4Fill" />
              </Button>
            </NavLink>
          )}
          {!SYSTEM_WIDGET_IDS.includes(widget.id) && (
            <Switch
              value={enabled}
              onChange={(value) => patchWidgetConfig(widget.id, { enabled: value })}
            />
          )}
        </>
      }
    />
  );
}

export function WidgetsView() {
  const { i18n } = useTranslation();
  const [search, setSearch] = useState("");

  const filtered = search
    ? widgets.value.filter((w) =>
      resolveDisplayName(w.metadata.displayName, i18n.language).toLowerCase().includes(search.toLowerCase())
    )
    : widgets.value;

  return (
    <>
      <ResourceListHeader
        discoverUrl="https://seelen.io/resources/s?category=Widget"
        search={search}
        onSearch={setSearch}
      />

      <div className={cs.list}>
        {filtered.map((widget) => <WidgetItem key={widget.id} widget={widget} />)}
      </div>
    </>
  );
}
