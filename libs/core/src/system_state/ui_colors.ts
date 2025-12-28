import { SeelenCommand, SeelenEvent, type UnSubscriber } from "../handlers/mod.ts";
import { RuntimeStyleSheet } from "../utils/DOM.ts";
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
    const oldStyles = new RuntimeStyleSheet("@deprecated/system-colors");
    const newStyles = new RuntimeStyleSheet("@runtime/system-colors");

    for (const [key, value] of Object.entries(this.inner)) {
      if (typeof value !== "string") {
        continue;
      }

      const color = Color.fromHex(value);
      const { r, g, b } = color;

      // replace rust snake case with kebab case
      const name = key.replace("_", "-");

      // @deprecated old names
      oldStyles.addVariable(`--config-${name}-color`, value.slice(0, 7));
      oldStyles.addVariable(`--config-${name}-color-rgb`, `${r}, ${g}, ${b}`);

      if (name.startsWith("accent")) {
        newStyles.addVariable(`--system-${name}-color`, value.slice(0, 7));
        newStyles.addVariable(`--system-${name}-color-rgb`, `${r}, ${g}, ${b}`);

        const complement = color.complementary();
        newStyles.addVariable(`--system-${name}-complementary-color`, complement.toHexString());
        newStyles.addVariable(
          `--system-${name}-complementary-color-rgb`,
          `${complement.r}, ${complement.g}, ${complement.b}`,
        );
      }
    }

    oldStyles.applyToDocument();
    newStyles.applyToDocument();
  }
}

export interface Color extends IColor {}

export class Color {
  constructor(obj: IColor) {
    this.r = obj.r;
    this.g = obj.g;
    this.b = obj.b;
    this.a = obj.a;
  }

  /** generates a random solid color */
  static random(): Color {
    return new Color({
      r: Math.floor(Math.random() * 255),
      g: Math.floor(Math.random() * 255),
      b: Math.floor(Math.random() * 255),
      a: 255,
    });
  }

  static fromHex(hex: string): Color {
    if (hex.startsWith("#")) {
      hex = hex.slice(1);
    }

    if (hex.length === 3) {
      hex = hex
        .split("")
        .map((char) => `${char}${char}`)
        .join("");
    }

    if (hex.length === 6) {
      hex = hex.padStart(8, "f");
    }

    const color = parseInt(hex.replace("#", ""), 16);
    return new Color({
      r: (color >> 24) & 255,
      g: (color >> 16) & 255,
      b: (color >> 8) & 255,
      a: color & 255,
    });
  }

  toHexString(): string {
    return (
      "#" +
      this.r.toString(16).padStart(2, "0") +
      this.g.toString(16).padStart(2, "0") +
      this.b.toString(16).padStart(2, "0") +
      (this.a === 255 ? "" : this.a.toString(16).padStart(2, "0"))
    );
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
      [`--color-${parsedName}-rgb`]: `${this.r}, ${this.g}, ${this.b}`,
      [`--color-${parsedName}-rgba`]: `${this.r}, ${this.g}, ${this.b}, ${this.a}`,
    });
  }

  /**
   * https://stackoverflow.com/questions/596216/formula-to-determine-perceived-brightness-of-rgb-color
   *
   * @param accuracy if true will use an expensive but more accurate algorithm
   * @returns a number between 0 and 255
   */
  calcLuminance(accuracy?: boolean): number {
    if (accuracy) {
      // gamma correction
      const gR = this.r ** 2.2;
      const gG = this.g ** 2.2;
      const gB = this.b ** 2.2;
      return (0.299 * gR + 0.587 * gG + 0.114 * gB) ** (1 / 2.2);
    }
    // standard algorithm
    return 0.2126 * this.r + 0.7152 * this.g + 0.0722 * this.b;
  }

  complementary(): Color {
    return new Color({
      r: 255 - this.r,
      g: 255 - this.g,
      b: 255 - this.b,
      a: this.a,
    });
  }
}
