import { AnimatedPopover } from "@shared/components/AnimatedWrappers";
import { Icon } from "libs/ui/react/components/Icon";
import { useWindowFocusChange } from "libs/ui/react/utils/hooks";
import { cx } from "libs/ui/react/utils/styling";
import moment from "moment";
import type { VNode } from "preact";
import { batch, useComputed, useSignal, useSignalEffect } from "@preact/signals";

import "./infra.css";
import { BackgroundByLayersV2 } from "libs/ui/react/components/BackgroundByLayers/infra";
import { $settings } from "../shared/state/mod.ts";
import { useCallback } from "preact/hooks";

type CalendarMode = "month" | "year";

interface CalendarHeaderProps {
  date: moment.Moment;
  viewMode: CalendarMode;
  onDateChange: (date: moment.Moment) => void;
  onViewModeChange: (mode: CalendarMode) => void;
}

function CalendarHeader({ date, viewMode, onDateChange, onViewModeChange }: CalendarHeaderProps) {
  const handlePrevious = () => {
    const newDate = date.clone().add(-1, viewMode === "month" ? "months" : "years");
    onDateChange(newDate);
  };

  const handleNext = () => {
    const newDate = date.clone().add(1, viewMode === "month" ? "months" : "years");
    onDateChange(newDate);
  };

  const handleToday = () => {
    onDateChange(moment().locale(date.locale()));
  };

  const toggleViewMode = () => {
    onViewModeChange(viewMode === "month" ? "year" : "month");
  };

  return (
    <div className="calendar-header">
      <span className="calendar-date" onClick={toggleViewMode}>
        {viewMode === "month" ? date.format("MMMM YYYY") : date.format("YYYY")}
      </span>
      <div className="calendar-actions">
        <button className="calendar-navigator" onClick={handlePrevious}>
          <Icon iconName="AiOutlineLeft" />
        </button>
        <button className="calendar-navigator" onClick={handleToday}>
          <Icon iconName="AiOutlineHome" />
        </button>
        <button className="calendar-navigator" onClick={handleNext}>
          <Icon iconName="AiOutlineRight" />
        </button>
      </div>
    </div>
  );
}

interface MonthViewProps {
  date: moment.Moment;
  selectedDate: moment.Moment;
  weekDays: string[];
  onDateSelect: (date: moment.Moment) => void;
}

function MonthView({ date, selectedDate, weekDays, onDateSelect }: MonthViewProps) {
  const today = moment();
  const startOfMonth = date.clone().startOf("month");
  const endOfMonth = date.clone().endOf("month");
  const startDate = startOfMonth.clone().startOf("week");
  const endDate = endOfMonth.clone().endOf("week");

  const weeks: moment.Moment[][] = [];
  let currentWeek: moment.Moment[] = [];
  let currentDate = startDate.clone();

  while (currentDate.isSameOrBefore(endDate, "day")) {
    currentWeek.push(currentDate.clone());
    if (currentWeek.length === 7) {
      weeks.push(currentWeek);
      currentWeek = [];
    }
    currentDate.add(1, "day");
  }

  return (
    <div className="calendar-month-view">
      <div className="calendar-weekdays">
        {weekDays.map((day, index) => (
          <div key={index} className="calendar-weekday">
            {day}
          </div>
        ))}
      </div>
      <div className="calendar-days">
        {weeks.map((week, weekIndex) => (
          <div key={weekIndex} className="calendar-week">
            {week.map((day, dayIndex) => {
              const isToday = day.isSame(today, "day");
              const isSelected = day.isSame(selectedDate, "day");
              const isOffMonth = day.month() !== date.month();

              return (
                <div
                  key={dayIndex}
                  className={cx("calendar-cell", {
                    "calendar-cell-today": isToday,
                    "calendar-cell-selected": isSelected,
                    "calendar-cell-off-month": isOffMonth,
                  })}
                  onClick={() => onDateSelect(day)}
                >
                  {day.format("D")}
                </div>
              );
            })}
          </div>
        ))}
      </div>
    </div>
  );
}

interface YearViewProps {
  date: moment.Moment;
  onMonthSelect: (date: moment.Moment) => void;
}

function YearView({ date, onMonthSelect }: YearViewProps) {
  const today = moment();
  const months: moment.Moment[] = [];

  for (let i = 0; i < 12; i++) {
    months.push(date.clone().month(i).startOf("month"));
  }

  return (
    <div className="calendar-year-view">
      {months.map((month, index) => {
        const isCurrentMonth = month.isSame(today, "month");

        return (
          <div
            key={index}
            className={cx("calendar-month-cell", {
              "calendar-month-cell-current": isCurrentMonth,
            })}
            onClick={() => onMonthSelect(month)}
          >
            {month.format("MMMM")}
          </div>
        );
      })}
    </div>
  );
}

function DateCalendar() {
  const $language = useComputed(() => $settings.value.language);
  const $start_of_week = useComputed(() => $settings.value.startOfWeek);

  const $date = useSignal(moment().locale($language.value));
  const $selectedDate = useSignal(moment().locale($language.value));
  const $viewMode = useSignal<CalendarMode>("month");
  const $weekDays = useSignal<string[]>([]);

  useSignalEffect(() => {
    // Map StartOfWeek enum to moment day numbers (0 = Sunday, 1 = Monday, 6 = Saturday)
    const startDayMap = {
      Sunday: 0,
      Monday: 1,
      Saturday: 6,
    };
    const startDay = startDayMap[$start_of_week.value];

    // Configure moment locale to use the selected start of week
    moment.updateLocale($language.value, {
      week: {
        dow: startDay,
      },
    });

    // Generate week days starting from the configured day
    const weekStart = moment().locale($language.value).startOf("week");
    const newShortWeekDays = Array.from({ length: 7 }, (_, i) => weekStart.clone().add(i, "days").format("dd"));

    batch(() => {
      $weekDays.value = newShortWeekDays;
      $date.value = $date.value.locale($language.value);
      $selectedDate.value = $selectedDate.value.locale($language.value);
    });
  });

  const onWheel = useCallback((e: WheelEvent) => {
    e.preventDefault();
    e.stopPropagation();

    const isUp = e.deltaY < 0;
    $date.value = $date.value
      .clone()
      .add(isUp ? 1 : -1, $viewMode.value === "month" ? "months" : "years");
  }, []);

  const handleDateSelect = (date: moment.Moment) => {
    batch(() => {
      $selectedDate.value = date;
      $date.value = date;
    });
  };

  const handleMonthSelect = (date: moment.Moment) => {
    batch(() => {
      $date.value = date;
      $viewMode.value = "month";
    });
  };

  return (
    <BackgroundByLayersV2
      className="calendar-container"
      prefix="calendar"
      onContextMenu={(e) => e.stopPropagation()}
    >
      <div className="calendar" onWheel={onWheel}>
        <CalendarHeader
          date={$date.value}
          viewMode={$viewMode.value}
          onDateChange={(date) => ($date.value = date)}
          onViewModeChange={(mode) => ($viewMode.value = mode)}
        />
        {$viewMode.value === "month"
          ? (
            <MonthView
              date={$date.value}
              selectedDate={$selectedDate.value}
              weekDays={$weekDays.value}
              onDateSelect={handleDateSelect}
            />
          )
          : <YearView date={$date.value} onMonthSelect={handleMonthSelect} />}
      </div>
    </BackgroundByLayersV2>
  );
}

export function WithDateCalendar({ children }: { children: VNode }) {
  const $openPreview = useSignal(false);

  useWindowFocusChange((focused) => {
    if (!focused) {
      $openPreview.value = false;
    }
  });

  return (
    <AnimatedPopover
      animationDescription={{
        openAnimationName: "calendar-open",
        closeAnimationName: "calendar-close",
      }}
      open={$openPreview.value}
      trigger="click"
      onOpenChange={(value) => ($openPreview.value = value)}
      content={<DateCalendar />}
    >
      {children}
    </AnimatedPopover>
  );
}
