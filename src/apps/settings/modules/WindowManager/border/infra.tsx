import { SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { InputNumber, Switch } from 'antd';
import { useTranslation } from 'react-i18next';

import { useAppDispatch, useAppSelector, useDispatchCallback } from '../../shared/utils/infra';

import { BorderSelectors } from '../../shared/store/app/selectors';
import { BorderActions } from './app';

export const BorderSettings = () => {
  const enabled = useAppSelector(BorderSelectors.enabled);
  const offset = useAppSelector(BorderSelectors.offset);
  const width = useAppSelector(BorderSelectors.width);

  const dispatch = useAppDispatch();
  const { t } = useTranslation();

  const toggleEnabled = useDispatchCallback((value: boolean) => {
    dispatch(BorderActions.setEnabled(value));
  });

  const updateOffset = useDispatchCallback((value: number | null) => {
    dispatch(BorderActions.setOffset(value || 0));
  });

  const updateWidth = useDispatchCallback((value: number | null) => {
    dispatch(BorderActions.setWidth(value || 0));
  });

  return (
    <SettingsSubGroup
      label={
        <SettingsOption>
          <span>{t('wm.border.enable')}</span>
          <Switch value={enabled} onChange={toggleEnabled} />
        </SettingsOption>
      }
    >
      <SettingsOption>
        <span>{t('wm.border.offset')}</span>
        <InputNumber value={offset} onChange={updateOffset} />
      </SettingsOption>
      <SettingsOption>
        <span>{t('wm.border.width')}</span>
        <InputNumber value={width} onChange={updateWidth} />
      </SettingsOption>
    </SettingsSubGroup>
  );
};
