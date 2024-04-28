import moment from 'moment';
import { useEffect, useState } from 'react';

export function DateModule() {
  const [date, setDate] = useState(moment().format('MMM Do, HH:mm'));

  useEffect(() => {
    const id = setInterval(() => {
      setDate(moment().format('MMM Do, HH:mm'));
    }, 60000);
    return () => clearInterval(id);
  }, []);

  return <div className="ft-bar-item">{date}</div>;
}