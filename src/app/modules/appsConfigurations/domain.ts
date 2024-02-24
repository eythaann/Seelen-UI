import { IdWithIdentifier as JsonIdWithIdentifier } from '../../../YamlSettings.interface';

import { Rect } from '../shared/app/Rect';

export enum ApplicationOptions {
  Float = 'float',
  Unmanage = 'unmanage',
  Force = 'force',
  ObjectNameChange = 'object_name_change',
  Layered = 'layered',
  /*BorderOverflow = 'border_overflow',*/ // present in komorebi but deprecated
  TrayAndMultiWindow = 'tray_and_multi_window',
}

export const LabelByAppOption: Record<ApplicationOptions, string> = {
  [ApplicationOptions.Float]: 'Float',
  /*[ApplicationOptions.BorderOverflow]: 'Border Overflow',*/
  [ApplicationOptions.Force]: 'Forced',
  [ApplicationOptions.Layered]: 'Layered',
  [ApplicationOptions.ObjectNameChange]: 'Name Change',
  [ApplicationOptions.TrayAndMultiWindow]: 'MultiWindow',
  [ApplicationOptions.Unmanage]: 'Unmanaged',
};

export enum ApplicationIdentifier {
  Exe = 'Exe',
  Class = 'Class',
  Title = 'Title',
}

export enum MatchingStrategy {
  Legacy = 'Legacy',
  Equals = 'Equals',
  StartsWith = 'StartsWith',
  EndsWith = 'EndsWith',
  Contains = 'Contains',
  Regex = 'Regex',
}

export interface IdWithIdentifier {
  kind: ApplicationIdentifier;
  id: string;
  matching_strategy: MatchingStrategy;
}

type AppConfigurationsOptions = { [K in ApplicationOptions]: boolean };
export interface AppConfiguration extends AppConfigurationsOptions {
  name: string;
  category: string | null;
  workspace: string | null;
  monitor: number | null;
  kind: ApplicationIdentifier;
  identifier: string;
  matchingStrategy: MatchingStrategy;
  invisibleBorders: Rect.plain | null;
}

export interface AppConfigurationExtended extends AppConfiguration {
  key: number;
  isTemplate?: boolean;
  templateName?: string;
  templateDescription?: string;
}

export class AppConfiguration {
  static default(): AppConfiguration {
    return {
      name: 'New App',
      category: null,
      workspace: null,
      monitor: null,
      kind: ApplicationIdentifier.Exe,
      identifier: 'new-app.exe',
      matchingStrategy: MatchingStrategy.Equals,
      [ApplicationOptions.Float]: false,
      /*[ApplicationOptions.BorderOverflow]: false,*/
      [ApplicationOptions.Force]: false,
      [ApplicationOptions.Layered]: false,
      [ApplicationOptions.ObjectNameChange]: false,
      [ApplicationOptions.TrayAndMultiWindow]: false,
      [ApplicationOptions.Unmanage]: false,
      invisibleBorders: null,
    };
  }

  static from(json_identifier: JsonIdWithIdentifier): AppConfiguration {
    return {
      ...AppConfiguration.default(),
      name: json_identifier.id,
      identifier: json_identifier.id,
      kind: json_identifier.kind as ApplicationIdentifier,
      matchingStrategy: json_identifier.matching_strategy as MatchingStrategy ?? MatchingStrategy.Legacy,
    };
  }
}
