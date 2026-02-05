const units = ["Kb", "Mb", "Gb", "Tb", "Pb", "Eb", "Zb", "Yb"];

let totalRecieved = networkStatistics.reduce((total, net) => total + net.received, 0) * 8;
let totalTransmitted = networkStatistics.reduce((total, net) => total + net.transmitted, 0) * 8;
let unit = "B";

units.forEach((unitSize) => {
  if (totalRecieved >= 1000 || totalTransmitted >= 1000) {
    totalRecieved /= 1000;
    totalTransmitted /= 1000;
    unit = unitSize;
  } else {
    return;
  }
});

return [
  icon("PiArrowsDownUpBold"),
  " ",
  totalRecieved.toFixed(0) + unit + "ps" + " | " + totalTransmitted.toFixed(0) + unit + "ps",
];
