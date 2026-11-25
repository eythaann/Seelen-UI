if (!batteries.length) {
  return Icon({ name: "TbPlugConnected" });
}

const averagePercentage = batteries.reduce((total, battery) => total + battery.percentage, 0) / batteries.length;

const isCharging = power.acLineStatus === 1 ||
  batteries.some((battery) => {
    return battery.state === "charging";
  });
const isSmartChargeActive = batteries.some((battery) => battery.smartCharging);

let group = [];

if (isCharging) {
  group.push(Icon({ name: "BsLightningChargeFill" }));
  if (isSmartChargeActive) {
    group.push(Icon({ name: "IoHeart" }));
  }
  group.push(" ");
}

if (powerMode === "BetterBattery" || powerMode === "BatterySaver") {
  group.push(Icon({ name: "IoLeaf" }));
  group.push(" ");
} else if (powerMode === "HighPerformance" || powerMode === "MaxPerformance") {
  group.push(Icon({ name: "IoSpeedometer" }));
  group.push(" ");
} else if (powerMode === "GameMode") {
  group.push(Icon({ name: "IoGameController" }));
  group.push(" ");
} else if (powerMode === "MixedReality") {
  group.push(Icon({ name: "TbCardboardsFilled" }));
  group.push(" ");
}

group.push(
  averagePercentage > 90
    ? Icon({ name: "PiBatteryFullFill" })
    : averagePercentage > 66
    ? Icon({ name: "PiBatteryHighFill" })
    : averagePercentage > 33
    ? Icon({ name: "PiBatteryMediumFill" })
    : averagePercentage > 5
    ? Icon({ name: "PiBatteryLowFill" })
    : Icon({ name: "PiBatteryWarning" }),
);

group.push(" ");
group.push(averagePercentage);
group.push("%");

return group;
