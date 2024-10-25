
import { Calendar, Row } from 'antd';
import moment, { Moment } from 'moment';
import momentGenerateConfig from 'rc-picker/es/generate/moment';
import { useState, WheelEvent } from 'react';
import { useTimeout } from 'seelen-core';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import { Icon } from '../../../../shared/components/Icon';

const MomentCalendar = Calendar.generateCalendar<Moment>(momentGenerateConfig);

export function DateCalendar() {
  const [ date, setDate ] = useState(moment());
  const [ dateDirection, setDateDirection ] = useState(0);

  useTimeout(() => {
    if (dateDirection === 0)
      return;

    // Set only one commulative value to changer.
    setDate(date.clone().startOf('month').add(dateDirection > 0 ? 1 : -1, 'months'));
    // Reset the throtle variable
    setDateDirection(0);
    //The time here is really essential. 50-300 is the variable set here, 400 is really frustrating,
    //and below 50 the throtle not work approprietly. Set the sensitivity if it is not appropriate.
  }, 150, [dateDirection]);

  // This handler is really frequently fired and has to throtled!
  const wheelEventHandler = function (e: WheelEvent<HTMLDivElement>) {
    if (e.deltaY !== 0) {
      e.preventDefault();
      e.stopPropagation();

      //Throtle handling got to change state everytime to not stuck and handle the change all time, when new event fired.
      setDateDirection(dateDirection + e.deltaY);
    }
  };

  return (
    <BackgroundByLayersV2 className="calendar-container" prefix="calendar" onWheel={wheelEventHandler}>
      <MomentCalendar
        value={date}
        onChange={setDate}
        className="calendar"
        fullscreen={false}
        headerRender={() => {
          return (
            <Row className="calendar-header">
              <span className="calendar-date">{date.format('MMMM YYYY')}</span>
              <div className="calendar-header-placeholder"/>
              <button className="calendar-navigator" onClick={() => setDate(date.clone().startOf('month').add(-1, 'months'))}><Icon iconName="AiOutlineLeft" /></button>
              <button className="calendar-navigator" onClick={() => setDate(moment().startOf('month'))}><Icon iconName="AiOutlineHome" /></button>
              <button className="calendar-navigator" onClick={() => setDate(date.clone().startOf('month').add(1, 'months'))}><Icon iconName="AiOutlineRight" /></button>
            </Row>
          );
        }}
      />
    </BackgroundByLayersV2>
  );
}
