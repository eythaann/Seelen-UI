import type { ThemeConfigDefinition as ThemeConfigDef, ThemeId } from "@seelen-ui/lib/types";
import { ResourceText } from "@shared/components/ResourceText";
import type { ReactNode } from "react";

import { SettingsGroup, SettingsSubGroup } from "../../../../components/SettingsBox/index.tsx";

import { ThemeSetting } from "./ThemeSetting.tsx";

export interface ThemeConfigDefinitionProps {
  def: ThemeConfigDef;
  themeId: ThemeId;
  nestLevel?: number;
}

export function ThemeConfigDefinition({
  def,
  themeId,
  nestLevel = 0,
}: ThemeConfigDefinitionProps) {
  const content = renderContent(def, themeId, nestLevel);

  return nestLevel === 0 ? <SettingsGroup>{content}</SettingsGroup> : content;
}

function renderContent(def: ThemeConfigDef, themeId: ThemeId, nestLevel: number): ReactNode {
  if ("group" in def) {
    return (
      <SettingsSubGroup label={<ResourceText text={def.group.header} />}>
        {def.group.items.map((item, idx) => (
          <ThemeConfigDefinition key={idx} themeId={themeId} def={item} nestLevel={nestLevel + 1} />
        ))}
      </SettingsSubGroup>
    );
  }

  return <ThemeSetting themeId={themeId} definition={def} />;
}
