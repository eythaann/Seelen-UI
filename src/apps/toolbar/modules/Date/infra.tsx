import { DateToolbarItem } from '@seelen-ui/lib/types';
import moment from 'moment';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { Item } from '../item/infra/infra';

import { Selectors } from '../shared/store/app';
import { useInterval } from 'src/apps/shared/hooks';

import { WithDateCalendar } from './Calendar';

interface Props {
  module: DateToolbarItem;
}

export function DateModule({ module }: Props) {
  const dateFormat = useSelector(Selectors.dateFormat);

  const { i18n } = useTranslation();

  const [date, setDate] = useState(moment().locale(i18n.language).format(dateFormat));

  let interval = dateFormat.includes('ss') ? 1000 : 1000 * 60;
  useInterval(
    () => {
      setDate(moment().locale(i18n.language).format(dateFormat));
    },
    interval,
    [dateFormat, i18n.language],
  );

  return (
    <WithDateCalendar>
      <Item extraVars={{ date }} module={module} />
    </WithDateCalendar>
  );
}
