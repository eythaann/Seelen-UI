import { Tooltip } from 'antd';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { newSelectors } from '../../modules/shared/store/app/reducer';

import { RoutePath } from '../navigation/routes';

export const RouteExtraInfo: { [key: string]: React.JSXElementConstructor<any> } = {
  [RoutePath.SettingsByApplication]: () => {
    const { t } = useTranslation();
    return (
      <Tooltip title={t('apps_configurations.extra_info')}>
        <span>🛈</span>
      </Tooltip>
    );
  },
  [RoutePath.AppLauncher]: () => {
    const shortcut = useSelector(newSelectors.ahkVariables.toggleLauncher);
    return (
      <span style={{ fontSize: '0.9rem', color: 'var(--color-gray-500)' }}>({shortcut.fancy})</span>
    );
  },
};
