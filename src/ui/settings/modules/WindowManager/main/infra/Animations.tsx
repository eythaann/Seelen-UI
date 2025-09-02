import { InputNumber, Select, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import {
  SettingsGroup,
  SettingsOption,
  SettingsSubGroup,
} from 'src/ui/settings/components/SettingsBox';

import { SeelenWmSelectors } from '../../../shared/store/app/selectors';
import { WManagerSettingsActions } from '../app';

export function WmAnimationsSettings() {
  const animations = useSelector(SeelenWmSelectors.animations);

  const { t } = useTranslation();
  const d = useDispatch();

  return (
    <SettingsGroup>
      <SettingsSubGroup
        label={
          <SettingsOption
            label={t('wm.animations.enable')}
            action={
              <Switch
                checked={animations.enabled}
                onChange={(value) => {
                  d(WManagerSettingsActions.setAnimations({ ...animations, enabled: value }));
                }}
              />
            }
          />
        }
      >
        <SettingsOption
          label={t('wm.animations.duration')}
          action={
            <InputNumber
              min={100}
              max={1500}
              value={Number(animations.durationMs)}
              onChange={(value) => {
                // TODO: the type is bigint but in reality it's a number, tis should be fixed on the types lib
                let parsed = (value || 100) as unknown as bigint;
                d(WManagerSettingsActions.setAnimations({ ...animations, durationMs: parsed }));
              }}
            />
          }
        />
        <SettingsOption
          label={t('wm.animations.ease_function')}
          action={
            <Select
              showSearch
              options={EaseFunctions}
              value={animations.easeFunction}
              onSelect={(value) => {
                d(WManagerSettingsActions.setAnimations({ ...animations, easeFunction: value }));
              }}
              style={{ width: '150px' }}
            />
          }
        />
      </SettingsSubGroup>
    </SettingsGroup>
  );
}

const EaseFunctions = [
  'Linear',
  'EaseIn',
  'EaseOut',
  'EaseInOut',
  'EaseInQuad',
  'EaseOutQuad',
  'EaseInOutQuad',
  'EaseInCubic',
  'EaseOutCubic',
  'EaseInOutCubic',
  'EaseInQuart',
  'EaseOutQuart',
  'EaseInOutQuart',
  'EaseInQuint',
  'EaseOutQuint',
  'EaseInOutQuint',
  'EaseInExpo',
  'EaseOutExpo',
  'EaseInOutExpo',
  'EaseInCirc',
  'EaseOutCirc',
  'EaseInOutCirc',
  'EaseInBack',
  'EaseOutBack',
  'EaseInOutBack',
  'EaseInElastic',
  'EaseOutElastic',
  'EaseInOutElastic',
  'EaseInBounce',
  'EaseOutBounce',
  'EaseInOutBounce',
].map((f) => ({ value: f }));
