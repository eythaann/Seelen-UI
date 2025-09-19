if (!batteries.length) {
  return null;
}

const averagePercentage = batteries.reduce((total, battery) => total + battery.percentage, 0) / batteries.length;

const isCharging = power.acLineStatus === 1 ||
  batteries.some((battery) => {
    return battery.state === "charging";
  });
const isSmartChargeActive = batteries.some((battery) => battery.smartCharging);

let group = [];

if (isCharging) {
  group.push(icon("BsFillLightningChargeFill", 12));
  group.push(" ");
}

if (powerMode === "BetterBattery" || powerMode === "BatterySaver") {
  group.push(icon("FaLeaf", 12));
  group.push(" ");
} else if (powerMode === "HighPerformance" || powerMode === "MaxPerformance") {
  group.push(icon("IoSpeedometer", 12));
  group.push(" ");
}

if (isSmartChargeActive) {
  group.push(icon("FaHeart", 12));
  group.push(" ");
}

group.push(
  averagePercentage > 90
    ? icon("PiBatteryFullFill")
    : averagePercentage > 66
    ? icon("PiBatteryHighFill")
    : averagePercentage > 33
    ? icon("PiBatteryMediumFill")
    : averagePercentage > 5
    ? icon("PiBatteryLowFill")
    : icon("PiBatteryWarning"),
);

group.push(" ");
group.push(averagePercentage);
group.push("%");

return group;
