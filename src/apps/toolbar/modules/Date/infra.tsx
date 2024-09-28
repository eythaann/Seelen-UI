import moment from 'moment';
import { useState } from 'react';
import { useSelector } from 'react-redux';
import { DateToolbarModule, useInterval } from 'seelen-core';

import { Item } from '../item/infra';

import { Selectors } from '../shared/store/app';

interface Props {
  module: DateToolbarModule;
}

export function DateModule({ module }: Props) {
  const dateFormat = useSelector(Selectors.dateFormat);

  const [date, setDate] = useState(moment().format(dateFormat));

  let interval = dateFormat.includes('ss') ? 1000 : 1000 * 60;
  useInterval(
    () => {
      setDate(moment().format(dateFormat));
    },
    interval,
    [dateFormat],
  );

  return <Item extraVars={{ date }} module={module} />;
}
