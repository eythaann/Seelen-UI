if (!batteries.length) {
  return t("plugged");
}

return batteries
  .map((battery, index) => {
    let content = "";

    if (batteries.length > 1) {
      content += `${index + 1}. ${battery.model}: `;
    }

    content += t("battery.remaining", { 0: battery.percentage });
    content += battery.smartCharging ? `- ${t("battery.smart_charge")}` : "";

    return content;
  })
  .join("\n");
