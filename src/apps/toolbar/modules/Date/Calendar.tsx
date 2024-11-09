import { Calendar, Popover } from 'antd';
import moment, { Moment } from 'moment';
import momentGenerateConfig from 'rc-picker/es/generate/moment';
import { PropsWithChildren, useCallback, useState, WheelEvent } from 'react';
import { useWindowFocusChange } from 'seelen-core';

import './infra.css';
import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';

const MomentCalendar = Calendar.generateCalendar<Moment>(momentGenerateConfig);

function DateCalendar() {
  const [date, setDate] = useState(moment());

  // Todo: this could be cool to be correctly implemented later
  const _onWheel = useCallback((e: WheelEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();

    const isUp = e.deltaY < 0;
    setDate((date) =>
      date
        .clone()
        .startOf('month')
        .add(isUp ? 1 : -1, 'months'),
    );
  }, []);

  return (
    <BackgroundByLayersV2 className="calendar-container" prefix="calendar">
      <MomentCalendar value={date} onChange={setDate} className="calendar" fullscreen={false} />
    </BackgroundByLayersV2>
  );
}

export function WithDateCalendar({ children }: PropsWithChildren) {
  const [openPreview, setOpenPreview] = useState(false);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  return (
    <Popover
      style={{ width: 300 }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={<DateCalendar />}
    >
      {children}
    </Popover>
  );
}
