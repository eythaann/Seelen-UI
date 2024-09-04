import { EventHandler, InvokeHandler, Obtainable } from '../handlers';

export interface UIColors {
  background: string;
  foreground: string;
  accent_darkest: string;
  accent_darker: string;
  accent_dark: string;
  accent: string;
  accent_light: string;
  accent_lighter: string;
  accent_lightest: string;
  complement: string | null;
}

export class UIColors extends Obtainable<UIColors>(
  InvokeHandler.GetSystemColors,
  EventHandler.UIColors,
) {
  static default(): UIColors {
    return {
      background: '#ffffff',
      foreground: '#000000',
      accent_darkest: '#990000',
      accent_darker: '#aa0000',
      accent_dark: '#bb0000',
      accent: '#cc0000',
      accent_light: '#dd0000',
      accent_lighter: '#ee0000',
      accent_lightest: '#ff0000',
      complement: null,
    };
  }

  static setAssCssVariables(colors: UIColors) {
    for (const [key, value] of Object.entries(colors)) {
      if (typeof value !== 'string') {
        continue;
      }
      let hex = value.replace('#', '').slice(0, 6);
      var color = parseInt(hex, 16);
      var r = (color >> 16) & 255;
      var g = (color >> 8) & 255;
      var b = color & 255;
      // replace rust snake case with kebab case
      let name = key.replace('_', '-');
      document.documentElement.style.setProperty(`--config-${name}-color`, value.slice(0, 7));
      document.documentElement.style.setProperty(`--config-${name}-color-rgb`, `${r}, ${g}, ${b}`);
    }
  }
}
