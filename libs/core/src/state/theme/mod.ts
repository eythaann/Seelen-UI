import type {
  ResourceId,
  Settings as ISettings,
  Theme as ITheme,
  ThemeConfigDefinition,
  ThemeId,
  ThemeVariableDefinition,
} from "@seelen-ui/types";
import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../../handlers/mod.ts";
import { List } from "../../utils/List.ts";
import { newFromInvoke, newOnEvent } from "../../utils/State.ts";
import { Widget } from "../widget/mod.ts";

export class ThemeList extends List<ITheme> {
  static getAsync(): Promise<ThemeList> {
    return newFromInvoke(this, SeelenCommand.StateGetThemes);
  }

  static onChange(cb: (payload: ThemeList) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.StateThemesChanged);
  }

  applyToDocument(activeIds: ThemeId[], variables: ISettings["byTheme"]): void {
    const enabledThemes: Theme[] = [];
    for (const theme of this.asArray()) {
      if (activeIds.includes(theme.id)) {
        enabledThemes.push(new Theme(theme));
      }
    }
    // sort by user order
    enabledThemes.sort((a, b) => activeIds.indexOf(a.id) - activeIds.indexOf(b.id));

    removeAllThemeStyles();
    for (const theme of enabledThemes) {
      theme.applyToDocument(variables[theme.id]);
    }
  }
}

export interface Theme extends ITheme {}

export class Theme {
  constructor(plain: ITheme) {
    Object.assign(this, plain);
  }

  forEachVariableDefinition(cb: (def: ThemeVariableDefinition) => void): void {
    iterateVariableDefinitions(this.settings, cb);
  }

  /** Will add the styles targeting the current widget id */
  applyToDocument(varValues: ISettings["byTheme"][ResourceId] = {}): void {
    const widgetId = Widget.getCurrentWidgetId();
    let styles = ``;

    this.forEachVariableDefinition((def) => {
      if (!isValidCssVariableName(def.name)) {
        return;
      }
      styles += `
        @property ${def.name} {
          syntax: "${def.syntax}";
          inherits: true;
          initial-value: ${def.initialValue}${"initialValueUnit" in def ? def.initialValueUnit : ""};
        }
      `;
    });

    const layerName = "theme-" +
      this.id
        .toLowerCase()
        .replaceAll("@", "")
        .replaceAll(/[^a-zA-Z0-9\-\_]/g, "_");

    styles += `@layer ${layerName}-shared {\n${this.sharedStyles}\n}\n`;

    const variablesContent = Object.entries(varValues)
      .filter(([name, _value]) => isValidCssVariableName(name))
      .map(([name, value]) => `${name}: ${value || ""};`)
      .join("\n");
    styles += `@layer ${layerName} {\n:root {${variablesContent}}\n${this.styles[widgetId] ?? ""}\n}\n`;

    this.removeFromDocument(); // remove old styles
    const styleElement = document.createElement("style");
    styleElement.id = this.id;
    styleElement.textContent = styles;
    styleElement.setAttribute("data-source", "theme");
    document.head.appendChild(styleElement);
  }

  removeFromDocument(): void {
    document.getElementById(this.id)?.remove();
  }
}

function isValidCssVariableName(name: string): boolean {
  return /^--[\w\d-]*$/.test(name);
}

function iterateVariableDefinitions(
  defs: ThemeConfigDefinition[],
  cb: (def: ThemeVariableDefinition) => void,
): void {
  for (const def of defs) {
    if ("group" in def) {
      iterateVariableDefinitions(def.group.items, cb);
    } else {
      cb(def);
    }
  }
}

function removeAllThemeStyles(): void {
  const elements = document.querySelectorAll(`style[data-source="theme"]`);
  for (const element of elements) {
    if (element instanceof HTMLStyleElement) {
      element.remove();
    }
  }
}
