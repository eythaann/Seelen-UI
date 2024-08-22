import { IdWithIdentifier } from '../../../shared/schemas/AppsConfigurations';

export enum WmApplicationOptions {
  Float = 'float',
  Unmanage = 'unmanage',
  ForceManage = 'force',
  Pinned = 'pinned',
}

export enum WegApplicationOptions {
  Hidden = 'hidden',
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

export interface AppConfiguration {
  name: string;
  category: string | null;
  workspace: string | null;
  monitor: number | null;
  identifier: IdWithIdentifier;
  isBundled: boolean;
  options: Array<WmApplicationOptions | WegApplicationOptions>;
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
      options: [],
    };
  }
}
