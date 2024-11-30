import { Calendar, Popover, Row } from 'antd';
import { CalendarMode } from 'antd/es/calendar/generateCalendar';
import moment, { Moment } from 'moment';
import momentGenerateConfig from 'rc-picker/es/generate/moment';
import { PropsWithChildren, useCallback, useState, WheelEvent } from 'react';
import { useWindowFocusChange } from 'seelen-core';

import './infra.css';
import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';

import { Icon } from '../../../shared/components/Icon';
import { cx } from '../../../shared/styles';

const MomentCalendar = Calendar.generateCalendar<Moment>(momentGenerateConfig);

function DateCalendar() {
  const [date, setDate] = useState(moment());
  const [viewMode, setViewMode] = useState<CalendarMode | undefined>('month');

  const onWheel = useCallback((e: WheelEvent<HTMLDivElement>) => {
    e.preventDefault();
    e.stopPropagation();

    const isUp = e.deltaY < 0;
    setDate((date) =>
      date
        .clone()
        .startOf('month')
        .add(isUp ? 1 : -1, viewMode as moment.unitOfTime.Base),
    );
  }, []);

  return (
    <BackgroundByLayersV2 className="calendar-container" prefix="calendar" onContextMenu={(e) => e.stopPropagation()}>
      <div onWheel={onWheel}>
        <MomentCalendar
          value={date}
          onChange={setDate}
          className="calendar"
          fullscreen={false}
          mode={viewMode}
          headerRender={(props) => props.type == 'month' ? (
            <Row className="calendar-header">
              <span className="calendar-date" onClick={() => setViewMode('year')}>{date.format('MMMM YYYY')}</span>
              <div className="calendar-header-placeholder"/>
              <button className="calendar-navigator" onClick={() => setDate(date.clone().startOf('month').add(-1, 'months'))}><Icon iconName="AiOutlineLeft" /></button>
              <button className="calendar-navigator" onClick={() => setDate(moment().startOf('month'))}><Icon iconName="AiOutlineHome" /></button>
              <button className="calendar-navigator" onClick={() => setDate(date.clone().startOf('month').add(1, 'months'))}><Icon iconName="AiOutlineRight" /></button>
            </Row>
          ) : (
            <Row className="calendar-header">
              <span className="calendar-date" onClick={() => setViewMode('month')}>{date.format('YYYY')}</span>
              <div className="calendar-header-placeholder"/>
              <button className="calendar-navigator" onClick={() => setDate(date.clone().startOf('year').add(-1, 'years'))}><Icon iconName="AiOutlineLeft" /></button>
              <button className="calendar-navigator" onClick={() => setDate(moment().startOf('month'))}><Icon iconName="AiOutlineHome" /></button>
              <button className="calendar-navigator" onClick={() => setDate(date.clone().startOf('year').add(1, 'years'))}><Icon iconName="AiOutlineRight" /></button>
            </Row>
          )}
          fullCellRender={(current, info) => info.type == 'date' ? (
            <div className={cx('calendar-cell-value', {
              'calendar-cell-today': current.isSame(info.today, 'date'),
              'calendar-cell-off-month': current.month() != date.month(),
            })}>{Number(current.format('DD'))}</div>
          ) : (
            <div className={cx('calendar-cell-value', 'calendar-cell-month', { 'calendar-cell-today': current.startOf('month').isSame(info.today.startOf('month'), 'date') })} onClick={() => {
              setDate(current);
              setViewMode('month');
            }}>{current.format('MMMM')}</div>
          )}/>
      </div>
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
