import { InputNumber } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../../shared/utils/infra';

import { SeelenWmSelectors } from '../../../shared/store/app/selectors';
import { WManagerSettingsActions } from '../app';

import { SettingsGroup, SettingsOption } from '../../../../components/SettingsBox';

export const OthersConfigs = () => {
  const resizeDelta = useAppSelector(SeelenWmSelectors.resizeDelta);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const onChangeResizeDelta = (value: number | null) => {
    dispatch(WManagerSettingsActions.setResizeDelta(value || 0));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <span>{t('wm.resize_delta')}</span>
          <InputNumber value={resizeDelta} onChange={onChangeResizeDelta} min={1} max={40} />
        </SettingsOption>
      </SettingsGroup>
    </>
  );
};
