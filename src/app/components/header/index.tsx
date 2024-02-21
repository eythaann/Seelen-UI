import { Button } from 'antd';

import { LoadSettingsToStore, SaveStore } from '../../modules/shared/infrastructure/store';

import { useAppSelector } from '../../modules/shared/app/hooks';
import { RootSelectors } from '../../modules/shared/app/selectors';

import { Route, RouteLabels } from '../../modules/shared/domain/routes';

import cs from './index.module.css';

export const Header = () => {
  let route = useAppSelector(RootSelectors.route);
  let hasChanges = useAppSelector(RootSelectors.toBeSaved);

  return (
    <div className={cs.Header}>
      {RouteLabels[route]}
      {route !== Route.INFO && (
        <div>
          <Button
            children="Cancel"
            type="default"
            danger
            disabled={!hasChanges}
            onClick={() => LoadSettingsToStore()}
          />
          {' '}
          <Button
            children="Save"
            type="primary"
            disabled={!hasChanges}
            onClick={SaveStore}
          />
        </div>
      )}
    </div>
  );
};
