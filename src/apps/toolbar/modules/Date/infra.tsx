import { DateToolbarModule, TimeUnit } from '../../../shared/schemas/Placeholders';
import moment from 'moment';
import { useEffect, useState } from 'react';

import { Item } from '../item/infra';

interface Props {
  module: DateToolbarModule;
}

const timeByUnit = {
  [TimeUnit.SECOND]: 1000,
  [TimeUnit.MINUTE]: 1000 * 60,
  [TimeUnit.HOUR]: 1000 * 60 * 60,
  [TimeUnit.DAY]: 1000 * 60 * 60 * 24,
};

export function DateModule({ module }: Props) {
  const [date, setDate] = useState(moment().format(module.format));

  useEffect(() => {
    const id = setInterval(() => {
      setDate(moment().format(module.format));
    }, timeByUnit[module.each]);
    return () => clearInterval(id);
  }, [module]);

  return <Item extraVars={{ date }} module={module} />;
}