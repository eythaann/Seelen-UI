import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../handlers/mod.ts";
import { newFromInvoke, newOnEvent } from "../utils/State.ts";
import type { Color as IColor, UIColors as IUIColors } from "@seelen-ui/types";

export class UIColors {
  constructor(public inner: IUIColors) {}

  static getAsync(): Promise<UIColors> {
    return newFromInvoke(this, SeelenCommand.SystemGetColors);
  }

  static onChange(cb: (payload: UIColors) => void): Promise<UnSubscriber> {
    return newOnEvent(cb, this, SeelenEvent.ColorsChanged);
  }

  static default(): UIColors {
    return new this({
      background: "#ffffff",
      foreground: "#000000",
      accent_darkest: "#990000",
      accent_darker: "#aa0000",
      accent_dark: "#bb0000",
      accent: "#cc0000",
      accent_light: "#dd0000",
      accent_lighter: "#ee0000",
      accent_lightest: "#ff0000",
      complement: null,
    });
  }

  setAsCssVariables(): void {
    const id = "system-ui-color-variables";
    document.getElementById(id)?.remove();
    const element = document.createElement("style");
    element.id = id;
    element.textContent = ":root {\n";

    for (const [key, value] of Object.entries(this.inner)) {
      if (typeof value !== "string") {
        continue;
      }
      const hex = value.replace("#", "").slice(0, 6);
      const color = parseInt(hex, 16);
      const r = (color >> 16) & 255;
      const g = (color >> 8) & 255;
      const b = color & 255;

      // replace rust snake case with kebab case
      const name = key.replace("_", "-");
      element.textContent += `--system-${name}-color: ${value.slice(0, 7)};\n`;
      element.textContent += `--system-${name}-color-rgb: ${r}, ${g}, ${b};\n`;

      // @deprecated old names
      element.textContent += `--config-${name}-color: ${value.slice(0, 7)};\n`;
      element.textContent += `--config-${name}-color-rgb: ${r}, ${g}, ${b};\n`;
    }

    document.head.appendChild(element);
  }
}

export class Color {
  constructor(public inner: IColor) {}

  /** generates a random solid color */
  static random(): Color {
    return new Color({
      r: Math.floor(Math.random() * 255),
      g: Math.floor(Math.random() * 255),
      b: Math.floor(Math.random() * 255),
      a: 255,
    });
  }

  private getRuntimeStyleSheet(): HTMLStyleElement {
    const styleId = "slu-lib-runtime-color-variables";
    let styleElement = document.getElementById(styleId) as HTMLStyleElement;
    if (!styleElement) {
      styleElement = document.createElement("style");
      styleElement.id = styleId;
      styleElement.textContent = ":root {\n}";
      document.head.appendChild(styleElement);
    }
    return styleElement;
  }

  private insertIntoStyleSheet(obj: Record<string, string>): void {
    const sheet = this.getRuntimeStyleSheet();
    const lines = sheet.textContent!.split("\n");
    lines.pop(); // remove the closing brace

    for (const [key, value] of Object.entries(obj)) {
      const old = lines.findIndex((line) => line.startsWith(key));
      if (old !== -1) {
        lines[old] = `${key}: ${value};`;
      } else {
        lines.push(`${key}: ${value};`);
      }
    }

    lines.push("}");
    sheet.textContent = lines.join("\n");
  }

  toHexString(): string {
    return (
      "#" +
      this.inner.r.toString(16).padStart(2, "0") +
      this.inner.g.toString(16).padStart(2, "0") +
      this.inner.b.toString(16).padStart(2, "0") +
      this.inner.a.toString(16).padStart(2, "0")
    );
  }

  /**
   * @param name the name of the color
   * the name will be parsed to lower kebab case and remove non-alphanumeric characters
   * this will create some css variables as:\
   * `--color-{name}` -> #RRGGBBAA\
   * `--color-{name}-rgb` -> R, G, B
   * `--color-{name}-rgba` -> R, G, B, A
   */
  setAsCssVariable(name: string): void {
    const parsedName = name
      .replace("_", "-")
      .replace(/[^a-zA-Z0-9\-]/g, "")
      .toLowerCase();

    this.insertIntoStyleSheet({
      [`--color-${parsedName}`]: this.toHexString(),
      [`--color-${parsedName}-rgb`]: `${this.inner.r}, ${this.inner.g}, ${this.inner.b}`,
      [`--color-${parsedName}-rgba`]: `${this.inner.r}, ${this.inner.g}, ${this.inner.b}, ${this.inner.a}`,
    });
  }

  /**
   * https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color
   *
   * @param accuracy if true will use an expensive but more accurate algorithm
   * @returns a number between 0 and 255
   */
  calcLuminance(accuracy?: boolean): number {
    const { r, g, b } = this.inner;
    if (accuracy) {
      // gamma correction
      const gR = r ** 2.2;
      const gG = g ** 2.2;
      const gB = b ** 2.2;
      return (0.299 * gR + 0.587 * gG + 0.114 * gB) ** (1 / 2.2);
    }
    // standard algorithm
    return 0.2126 * r + 0.7152 * g + 0.0722 * b;
  }
}
