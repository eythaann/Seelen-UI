import { useSelector } from 'react-redux';

import { Selectors } from '../../../shared/store/app';

import { Reservation } from '../../domain';

import { cx } from '../../../../../shared/styles';

export function ReservedContainer({ reservation }: { reservation: Reservation }) {
  const { floating } = useSelector(Selectors.settings);
  return (
    <div
      className={cx('wm-container', 'wm-reserved', `wm-reserved-${reservation.toLowerCase()}`)}
      style={
        reservation === Reservation.Float
          ? {
            width: floating.width,
            height: floating.height,
          }
          : undefined
      }
    />
  );
}
