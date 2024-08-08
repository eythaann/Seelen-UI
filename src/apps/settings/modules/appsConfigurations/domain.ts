import { IdWithIdentifier } from '../../../shared/schemas/AppsConfigurations';

export enum ApplicationOptions {
  Float = 'float',
  Unmanage = 'unmanage',
  ForceManage = 'force',
  Pinned = 'pinned',
}

export enum ApplicationIdentifier {
  Exe = 'Exe',
  Class = 'Class',
  Title = 'Title',
  Path = 'Path',
}

export enum MatchingStrategy {
  Legacy = 'Legacy',
  Equals = 'Equals',
  StartsWith = 'StartsWith',
  EndsWith = 'EndsWith',
  Contains = 'Contains',
  Regex = 'Regex',
}

type AppConfigurationsOptions = { [K in ApplicationOptions]: boolean };
export interface AppConfiguration extends AppConfigurationsOptions {
  name: string;
  category: string | null;
  workspace: string | null;
  monitor: number | null;
  identifier: IdWithIdentifier;
  isBundled: boolean;
}

export interface AppConfigurationExtended extends AppConfiguration {
  key: number;
}

export class AppConfiguration {
  static default(): AppConfiguration {
    return {
      name: 'New App',
      category: null,
      workspace: null,
      monitor: null,
      identifier: {
        id: 'new-app.exe',
        kind: ApplicationIdentifier.Exe,
        matchingStrategy: MatchingStrategy.Equals,
        negation: false,
        and: [],
        or: [],
      },
      isBundled: false,
      [ApplicationOptions.Float]: false,
      [ApplicationOptions.Unmanage]: false,
      [ApplicationOptions.Pinned]: false,
      [ApplicationOptions.ForceManage]: false,
    };
  }
}
