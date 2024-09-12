export enum AppExtraFlag {
  Float = 'float',
  Force = 'force',
  Unmanage = 'unmanage',
  Pinned = 'pinned',
  Hidden = 'hidden',
}

export enum AppIdentifierType {
  Exe = 'Exe',
  Class = 'Class',
  Title = 'Title',
  Path = 'Path',
}

export enum MatchingStrategy {
  Equals = 'Equals',
  StartsWith = 'StartsWith',
  EndsWith = 'EndsWith',
  Contains = 'Contains',
  Regex = 'Regex',
}

export interface AppIdentifier {
  id: string;
  kind: AppIdentifierType;
  matchingStrategy: MatchingStrategy;
  negation: boolean;
  and: AppIdentifier[];
  or: AppIdentifier[];
}

export class AppIdentifier {
  static placeholder(): AppIdentifier {
    return {
      id: 'new-app.exe',
      kind: AppIdentifierType.Exe,
      matchingStrategy: MatchingStrategy.Equals,
      negation: false,
      and: [],
      or: [],
    };
  }
}

export interface AppConfiguration {
  name: string;
  category: string | null;
  boundMonitor: number | null;
  boundWorkspace: string | null;
  identifier: AppIdentifier;
  options: Array<AppExtraFlag>;
  isBundled: boolean;
}

export class AppConfiguration {
  static placeholder(): AppConfiguration {
    return {
      name: 'New App',
      category: null,
      boundWorkspace: null,
      boundMonitor: null,
      identifier: AppIdentifier.placeholder(),
      isBundled: false,
      options: [],
    };
  }
}
