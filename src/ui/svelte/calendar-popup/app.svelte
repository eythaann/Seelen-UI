<script lang="ts">
  import { globalState } from "./state.svelte";
  import { Widget } from "@seelen-ui/lib";
  import Icon from "libs/ui/svelte/components/Icon/Icon.svelte";
  import moment from "moment";

  const today = moment();

  let weekDays = $derived.by(() => {
    const weekStart = moment().startOf("week");
    return Array.from({ length: 7 }, (_, i) =>
      weekStart.clone().add(i, "days").format("dd")
    );
  });

  function handlePrevious() {
    const newDate = globalState.date
      .clone()
      .add(-1, globalState.viewMode === "month" ? "months" : "years");
    globalState.date = newDate;
  }

  function handleNext() {
    const newDate = globalState.date
      .clone()
      .add(1, globalState.viewMode === "month" ? "months" : "years");
    globalState.date = newDate;
  }

  function handleToday() {
    globalState.date = moment().locale(globalState.date.locale());
  }

  function toggleViewMode() {
    globalState.viewMode = globalState.viewMode === "month" ? "year" : "month";
  }

  function handleDateSelect(day: moment.Moment) {
    globalState.selectedDate = day;
    globalState.date = day;
  }

  function handleMonthSelect(month: moment.Moment) {
    globalState.date = month;
    globalState.viewMode = "month";
  }

  function handleWheel(e: WheelEvent) {
    e.preventDefault();
    e.stopPropagation();

    const isUp = e.deltaY < 0;
    globalState.date = globalState.date
      .clone()
      .add(isUp ? 1 : -1, globalState.viewMode === "month" ? "months" : "years");
  }

  // Month view data
  const monthViewData = $derived.by(() => {
    const date = globalState.date;
    const selectedDate = globalState.selectedDate;
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

    return weeks;
  });

  // Year view data
  const yearViewData = $derived.by(() => {
    const date = globalState.date;
    const months: moment.Moment[] = [];

    for (let i = 0; i < 12; i++) {
      months.push(date.clone().month(i).startOf("month"));
    }

    return months;
  });

  $effect(() => {
    Widget.getCurrent().ready();
  });
</script>

<div class="slu-standard-popover calendar-popup">
  <div class="calendar" onwheel={handleWheel}>
    <!-- Calendar Header -->
    <div class="calendar-header">
      <span
        class="calendar-date"
        onclick={toggleViewMode}
        onkeydown={(e) => e.key === "Enter" && toggleViewMode()}
        role="button"
        tabindex="0"
      >
        {globalState.viewMode === "month"
          ? globalState.date.format("MMMM YYYY")
          : globalState.date.format("YYYY")}
      </span>
      <div class="calendar-actions">
        <button class="calendar-navigator" onclick={handlePrevious}>
          <Icon iconName="AiOutlineLeft" />
        </button>
        <button class="calendar-navigator" onclick={handleToday}>
          <Icon iconName="AiOutlineHome" />
        </button>
        <button class="calendar-navigator" onclick={handleNext}>
          <Icon iconName="AiOutlineRight" />
        </button>
      </div>
    </div>

    {#if globalState.viewMode === "month"}
      <!-- Month View -->
      <div class="calendar-month-view">
        <div class="calendar-weekdays">
          {#each weekDays as day}
            <div class="calendar-weekday">{day}</div>
          {/each}
        </div>
        <div class="calendar-days">
          {#each monthViewData as week}
            <div class="calendar-week">
              {#each week as day}
                {@const isToday = day.isSame(today, "day")}
                {@const isSelected = day.isSame(globalState.selectedDate, "day")}
                {@const isOffMonth = day.month() !== globalState.date.month()}
                <div
                  class="calendar-cell"
                  class:calendar-cell-today={isToday}
                  class:calendar-cell-selected={isSelected}
                  class:calendar-cell-off-month={isOffMonth}
                  onclick={() => handleDateSelect(day)}
                  onkeydown={(e) => e.key === "Enter" && handleDateSelect(day)}
                  role="button"
                  tabindex="0"
                >
                  {day.format("D")}
                </div>
              {/each}
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <!-- Year View -->
      <div class="calendar-year-view">
        {#each yearViewData as month}
          {@const isCurrentMonth = month.isSame(today, "month")}
          <div
            class="calendar-month-cell"
            class:calendar-month-cell-current={isCurrentMonth}
            onclick={() => handleMonthSelect(month)}
            onkeydown={(e) => e.key === "Enter" && handleMonthSelect(month)}
            role="button"
            tabindex="0"
          >
            {month.format("MMMM")}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
