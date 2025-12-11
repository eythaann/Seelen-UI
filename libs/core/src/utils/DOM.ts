export class RuntimeStyleSheet {
  #element: HTMLStyleElement;
  #variables: Array<[string, string]> = [];
  #styles: Array<string> = [];

  constructor(styleId: string) {
    let styleElement = document.getElementById(styleId) as HTMLStyleElement;
    if (!styleElement) {
      styleElement = document.createElement("style");
      styleElement.id = styleId;
      document.head.appendChild(styleElement);
    }
    this.#element = styleElement;
  }

  addVariable(key: string, value: string): void {
    this.#variables.push([key, value]);
  }

  addStyle(style: string): void {
    this.#styles.push(style);
  }

  clear(): void {
    this.#variables = [];
    this.#styles = [];
  }

  applyToDocument(): void {
    const vars = this.#variables.map(([key, value]) => `${key}: ${value};`).join("\n");
    this.#element.textContent = `:root {\n${vars}\n}\n\n`;
    this.#element.textContent += this.#styles.join("\n\n/* -=-=-=-=-=-=-=-=- */\n\n");
  }
}
