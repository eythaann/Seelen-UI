import { cx } from '../../../../../utils/styles';
import { useSelector } from 'react-redux';

import { Selectors } from '../../../shared/store/app';

import { Reservation } from '../../domain';

import cs from '../index.module.css';

export function ReservedContainer({ reservation }: { reservation: Reservation }) {
  const { floating } = useSelector(Selectors.settings);
  return (
    <div
      className={cx(cs.container, cs.reserved, cs[reservation.toLowerCase()])}
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
