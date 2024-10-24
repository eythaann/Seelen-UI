import { Popover } from 'antd';
import moment from 'moment';
import { useState } from 'react';
import { useSelector } from 'react-redux';
import { DateToolbarModule, useInterval, useWindowFocusChange } from 'seelen-core';

import { Item } from '../../item/infra/infra';

import { Selectors } from '../../shared/store/app';

import { DateCalendar } from './DateCalendar';

interface Props {
  module: DateToolbarModule;
}

export function DateModule({ module }: Props) {
  const dateFormat = useSelector(Selectors.dateFormat);

  const [openCalendar, setOpenCalendar] = useState(false);

  const [date, setDate] = useState(moment().format(dateFormat));

  let interval = dateFormat.includes('ss') ? 1000 : 1000 * 60;
  useInterval(
    () => {
      setDate(moment().format(dateFormat));
    },
    interval,
    [dateFormat],
  );

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenCalendar(false);
    }
  });

  return (<Popover
    open={openCalendar}
    trigger="click"
    onOpenChange={setOpenCalendar}
    arrow={false}
    content={<DateCalendar />}
  >
    <Item extraVars={{ date }} module={module} />
  </Popover>);
}
