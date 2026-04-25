if (!batteries.length) {
  return icon("TbPlugConnected");
}

const averagePercentage = batteries.reduce((total, battery) => total + battery.percentage, 0) / batteries.length;

const isCharging = power.acLineStatus === 1 ||
  batteries.some((battery) => {
    return battery.state === "charging";
  });
const isSmartChargeActive = batteries.some((battery) => battery.smartCharging);

let group = [];

if (isCharging) {
  group.push(icon("BsLightningChargeFill"));
  if (isSmartChargeActive) {
    group.push(icon("IoHeart"));
  }
  group.push(" ");
}

if (powerMode === "BetterBattery" || powerMode === "BatterySaver") {
  group.push(icon("IoLeaf"));
  group.push(" ");
} else if (powerMode === "HighPerformance" || powerMode === "MaxPerformance") {
  group.push(icon("IoSpeedometer"));
  group.push(" ");
} else if (powerMode === "GameMode") {
  group.push(icon("IoGameController"));
  group.push(" ");
} else if (powerMode === "MixedReality") {
  group.push(icon("TbCardboardsFilled"));
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
