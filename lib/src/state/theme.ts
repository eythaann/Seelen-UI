export interface ThemeCssByApp {
  /** Css Styles for the dock/taskbar */
  weg: string;
  /** Css Styles for the window manager */
  toolbar: string;
  /** Css Styles for the window manager */
  wm: string;
  /** Css Styles for the app launcher */
  launcher: string;
  /** Css Styles for the wall */
  wall: string;
}

export interface ThemeInfo {
  /** Display name of the theme */
  displayName: string;
  /** Author of the theme */
  author: string;
  /** Description of the theme */
  description: string;
  /** Filename of the theme, is overridden by the program on load */
  filename: string;
  /** Tags to be used in search */
  tags: string[];
}

export interface Theme {
  /** Metadata about the theme */
  info: ThemeInfo;
  /** Css Styles of the theme */
  styles: ThemeCssByApp;
}
