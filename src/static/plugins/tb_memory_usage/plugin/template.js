const units = ["KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];

let total = memory.total;
let used = memory.total - memory.free;
let unit = "B";

units.forEach((unitSize) => {
  if (total >= 1024) {
    total /= 1024;
    used /= 1024;
    unit = unitSize;
  } else {
    return;
  }
});

return [icon("FaMemory"), " ", used.toFixed(0) + unit + " / " + total.toFixed(0) + unit];
