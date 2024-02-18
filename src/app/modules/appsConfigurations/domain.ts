
export enum ApplicationOptions {
  Float = 'Float',
  Unmanage = 'Unmanage',
  Force = 'Force',
  ObjectNameChange = 'ObjectNameChange',
  Layered = 'Layered',
  BorderOverflow = 'BorderOverflow',
  TrayAndMultiWindow = 'TrayAndMultiWindow',
}
export enum ApplicationIdentifier {
  Exe = 'Exe',
  Class = 'Class',
  Title = 'Title',
}

export enum MatchingStrategy {
  Legacy = 'Legacy',
  Equals = 'Legacy',
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
  key: number;
  name: string;
  category: string | null;
  workspace: string | null;
  kind: ApplicationIdentifier;
  identifier: string;
  machingStrategy: MatchingStrategy;
}

export class AppConfiguration {
  static default(): AppConfiguration {
    return {
      key: 0,
      name: 'Fake App',
      category: null,
      workspace: null,
      kind: ApplicationIdentifier.Exe,
      identifier: 'FakeApp.exe',
      machingStrategy: MatchingStrategy.Legacy,
      [ApplicationOptions.Float]: false,
      [ApplicationOptions.BorderOverflow]: false,
      [ApplicationOptions.Force]: false,
      [ApplicationOptions.Layered]: false,
      [ApplicationOptions.ObjectNameChange]: false,
      [ApplicationOptions.TrayAndMultiWindow]: false,
      [ApplicationOptions.Unmanage]: false,
    };
  }
}