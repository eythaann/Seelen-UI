export function startDateCssVariables(): void {
  // Set initial values immediately
  updateDateCssVariables();

  // Update every minute (60000ms) to avoid overhead from seconds
  setInterval(updateDateCssVariables, 60000);
}

function updateDateCssVariables(): void {
  const now = new Date();
  const locale = navigator.language;

  // Time values
  const hour = now.getHours(); // 0-23
  const minute = now.getMinutes(); // 0-59

  // Date name values using Intl API
  const dayName = new Intl.DateTimeFormat(locale, { weekday: "long" }).format(now);
  const monthName = new Intl.DateTimeFormat(locale, { month: "long" }).format(now);

  // Date numeric values
  const dayOfMonth = now.getDate(); // 1-31
  const monthNumber = now.getMonth() + 1; // 1-12
  const year = now.getFullYear(); // 2025, etc.

  insertIntoStyleSheet({
    // Time variables
    "--date-hour": String(hour),
    "--date-minute": String(minute),
    // Date name variables (localized)
    "--date-day-name": dayName,
    "--date-month-name": monthName,
    // Date numeric variables
    "--date-day": String(dayOfMonth),
    "--date-month": String(monthNumber),
    "--date-year": String(year),
  });
}

function getRuntimeStyleSheet(): HTMLStyleElement {
  const styleId = "slu-lib-date-variables";
  let styleElement = document.getElementById(styleId) as HTMLStyleElement;
  if (!styleElement) {
    styleElement = document.createElement("style");
    styleElement.id = styleId;
    styleElement.textContent = ":root {\n}";
    document.head.appendChild(styleElement);
  }
  return styleElement;
}

function insertIntoStyleSheet(obj: Record<string, string>): void {
  const sheet = getRuntimeStyleSheet();
  const lines = sheet.textContent!.split("\n");
  lines.pop(); // remove the closing brace

  for (const [key, value] of Object.entries(obj)) {
    const old = lines.findIndex((line) => line.startsWith(key));
    if (old !== -1) {
      lines[old] = `${key}: ${value};`;
    } else {
      lines.push(`${key}: ${value};`);
    }
  }

  lines.push("}");
  sheet.textContent = lines.join("\n");
}
