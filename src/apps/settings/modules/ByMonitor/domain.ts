// This file is for testing, not final implementation yet.

interface WidgetConfigurationSwitch {
  type: 'switch';
  default: boolean;
}

interface WidgetConfigurationSelect {
  type: 'select';
  default: string | null;
  options: {
    value: string;
    label: string;
  }[];
}

export type WidgetConfigDeclarationItem = (WidgetConfigurationSwitch | WidgetConfigurationSelect) & {
  key: string;
  label: string;
  allowSetByMonitor?: boolean;
};

export interface WidgetSettingsDeclarationSubGroup {
  title: string;
  settings: WidgetConfigDeclarationItem[];
}

export interface WidgetSettingsDeclarationGroup {
  settings: Array<WidgetSettingsDeclarationSubGroup | WidgetConfigDeclarationItem>;
}
