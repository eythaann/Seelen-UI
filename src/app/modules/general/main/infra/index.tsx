import { SettingsBox } from '../../../../components/SettingsBox';

import { BorderSettings } from '../../border/infra';

export function General() {
  return <div>
    <BorderSettings />
    <SettingsBox>
      <div>test1</div>
      <div>test2</div>
    </SettingsBox>
    <SettingsBox>
      <div>test1</div>
      <div>test2</div>
    </SettingsBox>
    <SettingsBox>
      <div>test1</div>
      <div>test2</div>
    </SettingsBox>
    <SettingsBox>
      <div>test1</div>
      <div>test2</div>
    </SettingsBox>
    <SettingsBox>
      <div>test1</div>
      <div>test2</div>
    </SettingsBox>
    <SettingsBox>
      <div>test1</div>
      <div>test2</div>
    </SettingsBox>
  </div>;
}