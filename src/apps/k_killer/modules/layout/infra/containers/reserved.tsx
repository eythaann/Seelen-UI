import { cx } from '../../../../../utils/styles';

import { Reservation } from '../../domain';

import cs from '../index.module.css';

export function ReservedContainer({ reservation }: { reservation: Reservation }) {
  return (
    <div className={cx(cs.container, cs.reserved, cs[reservation.toLowerCase()])} />
  );
}