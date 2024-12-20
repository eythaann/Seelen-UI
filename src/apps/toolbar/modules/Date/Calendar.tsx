import { Calendar, Row } from 'antd';
import { CalendarMode, HeaderRender } from 'antd/es/calendar/generateCalendar';
import moment from 'moment';
import momentGenerateConfig from 'rc-picker/es/generate/moment';
import { PropsWithChildren, useCallback, useEffect, useState, WheelEvent } from 'react';
import { useTranslation } from 'react-i18next';

import './infra.css';
import { BackgroundByLayersV2 } from '../../../seelenweg/components/BackgroundByLayers/infra';

import { useWindowFocusChange } from 'src/apps/shared/hooks';

import { AnimatedPopover } from '../../../shared/components/AnimatedWrappers';
import { Icon } from '../../../shared/components/Icon';
import { cx } from '../../../shared/styles';

const short_week_days = {
  inner: ['Su', 'Mn', 'Tu', 'We', 'Th', 'Fr', 'Sa'],
};

const MomentCalendar = Calendar.generateCalendar({
  ...momentGenerateConfig,
  locale: {
    ...momentGenerateConfig.locale,
    getShortWeekDays: () => short_week_days.inner,
  },
});

const DateCalendarHeader: HeaderRender<moment.Moment> = (props) => {
  const { type, value: date, onChange, onTypeChange } = props;

  if (type === 'month') {
    return (
      <Row className="calendar-header">
        <span className="calendar-date" onClick={() => onTypeChange('year')}>
          {date.format('MMMM YYYY')}
        </span>
        <div className="calendar-actions">
          <button
            className="calendar-navigator"
            onClick={() => onChange(date.clone().add(-1, 'months'))}
          >
            <Icon iconName="AiOutlineLeft" />
          </button>
          <button
            className="calendar-navigator"
            onClick={() => onChange(moment().locale(date.locale()))}
          >
            <Icon iconName="AiOutlineHome" />
          </button>
          <button
            className="calendar-navigator"
            onClick={() => onChange(date.clone().add(1, 'months'))}
          >
            <Icon iconName="AiOutlineRight" />
          </button>
        </div>
      </Row>
    );
  }

  return (
    <Row className="calendar-header">
      <span className="calendar-date" onClick={() => onTypeChange('month')}>
        {date.format('YYYY')}
      </span>
      <div className="calendar-actions">
        <div className="calendar-header-placeholder" />
        <button
          className="calendar-navigator"
          onClick={() => onChange(date.clone().add(-1, 'years'))}
        >
          <Icon iconName="AiOutlineLeft" />
        </button>
        <button
          className="calendar-navigator"
          onClick={() => onChange(moment().locale(date.locale()))}
        >
          <Icon iconName="AiOutlineHome" />
        </button>
        <button
          className="calendar-navigator"
          onClick={() => onChange(date.clone().add(1, 'years'))}
        >
          <Icon iconName="AiOutlineRight" />
        </button>
      </div>
    </Row>
  );
};

function DateCalendar() {
  const { i18n } = useTranslation();

  const [date, setDate] = useState(moment().locale(i18n.language));
  const [viewMode, setViewMode] = useState<CalendarMode | undefined>('month');

  useEffect(() => {
    setDate(date.locale(i18n.language));
    const start = date.clone().startOf('isoWeek');
    short_week_days.inner = [
      start.day(0).format('dd'),
      start.day(1).format('dd'),
      start.day(2).format('dd'),
      start.day(3).format('dd'),
      start.day(4).format('dd'),
      start.day(5).format('dd'),
      start.day(6).format('dd'),
      start.day(7).format('dd'),
    ];
  }, [i18n.language]);

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
    <BackgroundByLayersV2
      className="calendar-container"
      prefix="calendar"
      onContextMenu={(e) => e.stopPropagation()}
    >
      <div onWheel={onWheel}>
        <MomentCalendar
          value={date}
          onChange={setDate}
          onPanelChange={(_, mode) => setViewMode(mode)}
          className="calendar"
          fullscreen={false}
          mode={viewMode}
          headerRender={DateCalendarHeader}
          fullCellRender={(current, info) =>
            info.type == 'date' ? (
              <div
                className={cx('calendar-cell-value', {
                  'calendar-cell-selected': current.isSame(date, 'date'),
                  'calendar-cell-today': current.isSame(info.today, 'date'),
                  'calendar-cell-off-month': current.month() != date.month(),
                })}
                onClick={() => setDate(current)}
              >
                {Number(current.format('DD'))}
              </div>
            ) : (
              <div
                className={cx('calendar-cell-value', 'calendar-cell-month', {
                  'calendar-cell-today': current
                    .startOf('month')
                    .isSame(info.today.startOf('month'), 'date'),
                })}
                onClick={() => {
                  setDate(current);
                  setViewMode('month');
                }}
              >
                {current.format('MMMM')}
              </div>
            )
          }
        />
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
    <AnimatedPopover
      animationDescription={{
        maxAnimationTimeMs: 500,
        openAnimationName: 'calendar-open',
        closeAnimationName: 'calendar-close',
      }}
      style={{ width: 300 }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={<DateCalendar />}
    >
      {children}
    </AnimatedPopover>
  );
}
