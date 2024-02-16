import { useAppSelector } from '../../modules/shared/app/hooks';
import { selectRoute } from '../../modules/shared/app/selectors';

import { RouteLabels } from '../../modules/shared/domain/routes';

import cs from './index.module.css';

export const Header = () => {
  let route = useAppSelector(selectRoute);

  return <div className={cs.Header}>
    {RouteLabels[route]}
  </div>;
};