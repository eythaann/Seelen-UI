import { invoke, SeelenCommand } from '../handlers';
import { Placeholder } from './placeholder';

export interface ProfileSettings {
  themes: string[];
}

export interface Profile {
  name: string;
  toolbarLayour: Placeholder;
  settings: ProfileSettings;
}

export class ProfileList {
  private constructor(private inner: Profile[]) {}

  static async getAsync(): Promise<ProfileList> {
    return new ProfileList(await invoke(SeelenCommand.stateGetProfiles));
  }

  toArray(): Profile[] {
    return this.inner;
  }
}