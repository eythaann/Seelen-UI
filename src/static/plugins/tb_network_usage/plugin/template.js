const units = ["Kb", "Mb", "Gb", "Tb", "Pb", "Eb", "Zb", "Yb"];

let totalRecieved = networkStatistics.reduce((total, net) => total + net.received, 0) * 8;
let totalTransmitted = networkStatistics.reduce((total, net) => total + net.transmitted, 0) * 8;
let unit = "b";

units.forEach((unitSize) => {
  if (totalRecieved >= 1000 || totalTransmitted >= 1000) {
    totalRecieved /= 1000;
    totalTransmitted /= 1000;
    unit = unitSize;
  } else {
    return;
  }
});

const recieved = totalRecieved === 0 ? "-" : totalRecieved.toFixed(0) + unit + "ps";
const transmitted = totalTransmitted === 0 ? "-" : totalTransmitted.toFixed(0) + unit + "ps";

return [icon("PiArrowsDownUpBold"), " ", recieved + " | " + transmitted];
