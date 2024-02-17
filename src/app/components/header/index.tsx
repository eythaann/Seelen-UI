import { Button } from 'antd';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../modules/shared/app/hooks';
import { RootActions } from '../../modules/shared/app/reducer';
import { RootSelectors } from '../../modules/shared/app/selectors';

import { Route, RouteLabels } from '../../modules/shared/domain/routes';

import cs from './index.module.css';

export const Header = () => {
  let route = useAppSelector(RootSelectors.route);
  let hasChanges = useAppSelector(RootSelectors.toBeSaved);

  let dispatch = useDispatch();

  let onSave = () => {
    dispatch(RootActions.setSaved());
  };

  let onCancel = () => {
    dispatch(RootActions.setSaved());
  };

  return (
    <div className={cs.Header}>
      {RouteLabels[route]}
      {route !== Route.INFO && (
        <div>
          <Button
            children="Cancel"
            type="default"
            danger
            size="small"
            disabled={!hasChanges}
            onClick={onCancel}
          />
          {' '}
          <Button
            children="Save"
            type="primary"
            size="small"
            disabled={!hasChanges}
            onClick={onSave}
          />
        </div>
      )}
    </div>
  );
};
