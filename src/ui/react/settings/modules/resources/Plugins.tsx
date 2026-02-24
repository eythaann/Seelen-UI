import { type Plugin, ResourceKind } from "@seelen-ui/lib/types";
import { ResourceText } from "libs/ui/react/components/ResourceText/index.tsx";
import { useTranslation } from "react-i18next";
import { useState } from "preact/hooks";
import React from "react";

import cs from "./infra.module.css";

import { plugins, widgets } from "../../state/resources.ts";

import { resolveDisplayName, ResourceCard, ResourceListHeader } from "./ResourceCard.tsx";

export function PluginsView() {
  const { i18n } = useTranslation();
  const [search, setSearch] = useState("");

  function targetLabel(target: string) {
    const widget = widgets.value.find((w) => w.id === target);
    if (widget) {
      return <ResourceText text={widget.metadata.displayName} />;
    }
    return <span>{target}</span>;
  }

  const filtered = search
    ? plugins.value.filter((p) =>
      resolveDisplayName(p.metadata.displayName, i18n.language).toLowerCase().includes(search.toLowerCase())
    )
    : plugins.value;

  const groupedByTarget = filtered.reduce((acc, plugin) => {
    acc[plugin.target] ??= {
      label: targetLabel(plugin.target),
      plugins: [],
    };
    acc[plugin.target]!.plugins.push(plugin);
    return acc;
  }, {} as Record<string, { label: React.ReactNode; plugins: Plugin[] }>);

  Object.values(groupedByTarget).forEach((group) => {
    group.plugins.sort((a, b) => a.id.localeCompare(b.id));
  });

  return (
    <>
      <ResourceListHeader
        discoverUrl="https://seelen.io/resources/s?category=Plugin"
        search={search}
        onSearch={setSearch}
      />

      <div className={cs.list}>
        {Object.values(groupedByTarget).map((group, idx) => (
          <React.Fragment key={idx}>
            <b>{group.label}</b>
            {group.plugins.map((plugin) => (
              <ResourceCard
                key={plugin.id}
                resource={plugin}
                kind={ResourceKind.Plugin}
              />
            ))}
          </React.Fragment>
        ))}
      </div>
    </>
  );
}
