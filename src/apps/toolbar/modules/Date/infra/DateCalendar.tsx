import { DayPicker } from 'react-day-picker';

import { BackgroundByLayersV2 } from '../../../../seelenweg/components/BackgroundByLayers/infra';

import 'react-day-picker/style.css';

export function DateCalendar() {
  return (
    <BackgroundByLayersV2 className="calendar-container" prefix="calendar">
      <DayPicker
        classNames={{
          root: 'calendar rdp-root',
        }}
        showWeekNumber={true}
      />
    </BackgroundByLayersV2>
  );
}
