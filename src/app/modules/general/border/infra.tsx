import { SettingsBox } from '../../../components/SettingsBox';

export const BorderSettings = () => {
  return <SettingsBox>
    <div>
      <div><span>Enable border</span></div>
      <div><span>Border offset</span></div>
      <div><span>Border width</span></div>
      <div><span>Border color</span></div>
    </div>
  </SettingsBox>;
};