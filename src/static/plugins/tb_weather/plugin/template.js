const current = weather.snapshot;
const options = weather.settings;

if (!options.useSystemLocation && !options.location) {
  return [icon("LuCloud"), t("weather.configure")];
}
if (weather.loading && !current) return icon("LuCloud");
if (!current) return [icon("LuCloud"), t("weather.unavailable")];

const ok = (v) => typeof v === "number" && Number.isFinite(v);
const num = (v, digits = 0) => (ok(v) ? String(digits ? Number(v.toFixed(digits)) : Math.round(v)) : null);
const time = (v) => (typeof v === "string" && v.length >= 16 ? v.substring(11, 16) : null);
const labeled = (key, value, unit = "") => (value === null ? null : `${t(key)} ${value}${unit}`);

const details = [
  options.showLocation ? current.location || t("weather.current_location") : null,
  options.showTemperature && ok(current.temperature) ? `${num(current.temperature)}°` : null,
  options.showCondition && current.condition ? t(`weather.conditions.${current.condition}`) : null,
  options.showRainChance ? labeled("weather.rain_short", num(current.rainChance), "%") : null,
  options.showPrecipitation
    ? labeled("weather.precipitation_short", num(current.precipitation, 1), ` ${options.precipitationUnit}`)
    : null,
  options.showWindSpeed ? labeled("weather.wind_short", num(current.windSpeed), ` ${options.windSpeedUnit}`) : null,
  options.showWindGusts ? labeled("weather.wind_gusts_short", num(current.windGusts), ` ${options.windSpeedUnit}`) : null,
  options.showWindDirection ? labeled("weather.wind_direction_short", num(current.windDirection), "°") : null,
  options.showHumidity ? labeled("weather.humidity_short", num(current.humidity), "%") : null,
  options.showCloudCover ? labeled("weather.cloud_cover_short", num(current.cloudCover), "%") : null,
  options.showPressure ? labeled("weather.pressure_short", num(current.pressure, options.pressureUnit === "inhg" ? 2 : 0), options.pressureUnit === "inhg" ? " inHg" : " hPa") : null,
  options.showUvIndex ? labeled("weather.uv_index_short", num(current.uvIndex)) : null,
  options.showSunriseSunset ? labeled("weather.sunrise_short", time(current.sunrise)) : null,
  options.showSunriseSunset ? labeled("weather.sunset_short", time(current.sunset)) : null,
  options.showMoonPhase && current.moonPhase
    ? `${t(`weather.moon.${current.moonPhase.name}`)}${ok(current.moonPhase.illumination) ? ` ${num(current.moonPhase.illumination)}%` : ""}`
    : null,
].filter(Boolean);

return details.length ? [icon("LuCloud"), details.join("\u2003")] : icon("LuCloud");