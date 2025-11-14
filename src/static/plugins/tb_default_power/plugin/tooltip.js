if (!batteries.length) {
  return "Plugged in";
}

return Group({
  style: {
    display: "flex",
    flexDirection: "column",
    gap: "4px",
  },
  content: batteries.map((battery, index) => {
    const content = [];
    if (batteries.length > 1) {
      content.push(`${index + 1}. ${battery.model}: `);
    }
    content.push(
      battery.percentage,
      t("placeholder.battery_remaining"),
      battery.smartCharging ? t("placeholder.smart_charge") : "",
    );
    // https://github.com/nyariv/SandboxJS/issues/29
    return Group({ content: content });
  }),
});
