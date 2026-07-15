if (!batteries.length) {
  return "Plugged in";
}

const batteriesText = batteries
  .map((battery, index) => {
    let content = batteries.length > 1 ? `${index + 1}. ` : "";
    content += `${Math.round(battery.percentage)}%`;
    if (battery.state === "charging") {
      content += " - Charging";
    }
    if (battery.smartCharging) {
      content += " - Smart charge";
    }
    return content;
  })
  .join("\n");

return batteriesText;
