if (activeSessionTime.isIdle) {
  return "Zzz";
}

const totalMinutes = Math.floor(activeSessionTime.activeSeconds / 60);
const hours = Math.floor(totalMinutes / 60);
const minutes = totalMinutes % 60;

return [icon("LuClock"), " ", hours ? `${hours}h ${minutes}m` : `${minutes}m`];
