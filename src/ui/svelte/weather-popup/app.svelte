<script lang="ts">
  import { Widget } from "@seelen-ui/lib";
  import {
    degreesToCardinal,
    weatherModule,
    type WeatherSettings,
  } from "libs/ui/svelte/utils/weather.svelte.ts";
  import { t } from "./i18n";

  let locationDraft = $state(weatherModule.settings.location);
  let settingsError = $state("");
  let openSections = $state<Record<string, boolean>>({ display: false, units: false });

  function toggleSection(name: string): void {
    openSections[name] = !openSections[name];
  }

  $effect(() => {
    locationDraft = weatherModule.settings.location;
  });

  const displayOptions: [keyof WeatherSettings, string][] = [
    ["showLocation", "show_location"],
    ["showTemperature", "show_temperature"],
    ["showCondition", "show_condition"],
    ["showRainChance", "show_rain_chance"],
    ["showPrecipitation", "show_precipitation"],
    ["showWindSpeed", "show_wind_speed"],
    ["showWindGusts", "show_wind_gusts"],
    ["showWindDirection", "show_wind_direction"],
    ["showHumidity", "show_humidity"],
    ["showCloudCover", "show_cloud_cover"],
    ["showPressure", "show_pressure"],
    ["showUvIndex", "show_uv_index"],
    ["showSunriseSunset", "show_sunrise_sunset"],
    ["showMoonPhase", "show_moon_phase"],
  ];

  function formatNumber(value: number | null | undefined): string {
    if (typeof value !== "number" || !Number.isFinite(value)) return "—";
    const locale = weatherModule.settings.locale === "system"
      ? undefined
      : weatherModule.settings.locale;
    return new Intl.NumberFormat(locale, { maximumFractionDigits: 1 }).format(value);
  }

  function formatTime(value: string | null | undefined): string {
    return value ? (value.split("T").at(-1)?.slice(0, 5) ?? value) : "—";
  }

  function withSuffix(value: number | null | undefined, suffix: string): string {
    const formatted = formatNumber(value);
    return formatted === "—" ? formatted : `${formatted}${suffix}`;
  }

  async function updateSettings(changes: Partial<WeatherSettings>): Promise<void> {
    settingsError = "";
    try {
      await weatherModule.updateSettings(changes);
      locationDraft = weatherModule.settings.location;
    } catch (error) {
      settingsError = String(error);
    }
  }

  async function submitLocation(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    await updateSettings({ location: locationDraft.trim(), useSystemLocation: false });
  }

  $effect(() => {
    const stop = weatherModule.activate();
    Widget.getCurrent().ready();
    return stop;
  });
</script>

<div class="slu-std-popover weather-popup">
  <header class="weather-header">
    <div>
      <div class="weather-title">{$t("title")}</div>
      <div class="weather-location">
        {weatherModule.snapshot?.location ||
          (weatherModule.settings.useSystemLocation ? $t("current_location") : $t("configure_location"))}
      </div>
    </div>
    <button
      type="button"
      data-skin="transparent"
      aria-label={$t("refresh")}
      title={$t("refresh")}
      disabled={weatherModule.loading}
      onclick={() => weatherModule.refresh()}
    >↻</button>
  </header>

  <section class="weather-section weather-location-section">
    <form class="weather-location-form" onsubmit={submitLocation}>
      <label>
        <span>{$t("location")}</span>
        <input
          type="search"
          data-skin="default"
          bind:value={locationDraft}
          placeholder={$t("location_placeholder")}
        />
      </label>
      <button type="submit" data-skin="default">{$t("save_location")}</button>
    </form>
    <label class="weather-setting">
      <span>{$t("use_system_location")}</span>
      <input
        type="checkbox"
        data-skin="switch"
        checked={weatherModule.settings.useSystemLocation}
        onchange={(event) =>
          updateSettings({ useSystemLocation: event.currentTarget.checked })}
      />
    </label>
    <div class="weather-secondary">{$t("source_note")}</div>
  </section>

  {#if weatherModule.loading && !weatherModule.snapshot}
    <div class="weather-message">{$t("loading")}</div>
  {:else if weatherModule.error}
    <div class="weather-error" role="alert">
      {weatherModule.error.startsWith("location.")
        ? $t(weatherModule.error)
        : weatherModule.error}
    </div>
  {:else if !weatherModule.snapshot}
    <div class="weather-message">{$t("no_weather")}</div>
  {:else}
    {@const current = weatherModule.snapshot}
    <section class="weather-current">
      <div class="weather-current-summary">
        <div class="weather-temperature">
          {formatNumber(current.temperature)}°
        </div>
        <div>
          <div>{$t(`conditions.${current.condition}`)}</div>
          <div class="weather-secondary">
            {$t("feels_like")}: {formatNumber(current.feelsLike)}°
          </div>
          <div class="weather-secondary">
            {$t("high_low")}: {formatNumber(current.high)}° / {formatNumber(current.low)}°
          </div>
        </div>
      </div>
      <div class="weather-stats">
        <div><span>{$t("rain_chance")}</span><strong>{withSuffix(current.rainChance, "%")}</strong></div>
        <div><span>{$t("precipitation")}</span><strong>{withSuffix(current.precipitation, ` ${weatherModule.settings.precipitationUnit}`)}</strong></div>
        <div><span>{$t("wind")}</span><strong>{withSuffix(current.windSpeed, ` ${weatherModule.settings.windSpeedUnit}`)}</strong></div>
        <div><span>{$t("wind_gusts")}</span><strong>{withSuffix(current.windGusts, ` ${weatherModule.settings.windSpeedUnit}`)}</strong></div>
        <div><span>{$t("wind_direction")}</span><strong>{current.windCardinal || degreesToCardinal(current.windDirection)}</strong></div>
        <div><span>{$t("humidity")}</span><strong>{withSuffix(current.humidity, "%")}</strong></div>
        <div><span>{$t("cloud_cover")}</span><strong>{withSuffix(current.cloudCover, "%")}</strong></div>
        <div><span>{$t("pressure")}</span><strong>{withSuffix(current.pressure, weatherModule.settings.pressureUnit === "inhg" ? " inHg" : " hPa")}</strong></div>
        <div><span>{$t("uv_index")}</span><strong>{formatNumber(current.uvIndex)}</strong></div>
        <div><span>{$t("sunrise")}</span><strong>{formatTime(current.sunrise)}</strong></div>
        <div><span>{$t("sunset")}</span><strong>{formatTime(current.sunset)}</strong></div>
        <div>
          <span>{$t("moon_phase")}</span>
          <strong>{$t(`moon.${current.moonPhase.name}`)} · {current.moonPhase.illumination}%</strong>
        </div>
      </div>
      <div class="weather-hourly">
        <strong>{$t("hourly_forecast")}</strong>
        {#each current.nextHours as hour}
          <div class="weather-hour">
            <span>{formatTime(hour.time)}</span>
            <span>{$t(`conditions.${hour.condition}`)}</span>
            <strong>{formatNumber(hour.temperature)}°</strong>
            <span>{withSuffix(hour.rainChance, "%")}</span>
          </div>
        {/each}
      </div>
      <div class="weather-secondary">
        {$t("updated")}: {new Intl.DateTimeFormat(
          weatherModule.settings.locale === "system" ? undefined : weatherModule.settings.locale,
          { timeStyle: "short" },
        ).format(current.updatedAt)}
      </div>
    </section>
  {/if}

  <section class="weather-section">
    <button
      type="button"
      class="weather-section-toggle"
      data-skin="transparent"
      aria-expanded={!!openSections.display}
      onclick={() => toggleSection("display")}
    >{$t("taskbar_display")}<span aria-hidden="true">{openSections.display ? "▾" : "▸"}</span></button>
    {#if openSections.display}
    <div class="weather-setting-list">
      <label class="weather-setting">
        <span>{$t("location_display_mode")}</span>
        <select
          data-skin="default"
          value={weatherModule.settings.locationDisplayMode || "town_and_state"}
          onchange={(event) =>
            updateSettings({
              locationDisplayMode: event.currentTarget.value as WeatherSettings["locationDisplayMode"],
            })}
        >
          <option value="town_and_state">{$t("town_and_state")}</option>
          <option value="town">{$t("town_only")}</option>
          <option value="state">{$t("state_only")}</option>
        </select>
      </label>
      {#each displayOptions as [key, label]}
        <label class="weather-setting">
          <span>{$t(label)}</span>
          <input
            type="checkbox"
            data-skin="switch"
            checked={weatherModule.settings[key] as boolean}
            onchange={(event) => updateSettings({ [key]: event.currentTarget.checked })}
          />
        </label>
      {/each}
    </div>
    {/if}
  </section>
  <section class="weather-section">
    <button
      type="button"
      class="weather-section-toggle"
      data-skin="transparent"
      aria-expanded={!!openSections.units}
      onclick={() => toggleSection("units")}
    >{$t("units_region")}<span aria-hidden="true">{openSections.units ? "▾" : "▸"}</span></button>
    {#if openSections.units}
    <div class="weather-setting-list">
    <label class="weather-setting">
      <span>{$t("regional_format")}</span>
      <select
        data-skin="default"
        value={weatherModule.settings.locale}
        onchange={(event) => updateSettings({ locale: event.currentTarget.value })}
      >
        <option value="system">{$t("system_locale")}</option>
        <option value="en-US">English (United States)</option>
        <option value="en-GB">English (United Kingdom)</option>
        <option value="fr-FR">Français (France)</option>
        <option value="de-DE">Deutsch (Deutschland)</option>
        <option value="es-ES">Español (España)</option>
      </select>
    </label>
    <label class="weather-setting">
      <span>{$t("temperature_unit")}</span>
      <select
        data-skin="default"
        value={weatherModule.settings.temperatureUnit}
        onchange={(event) =>
          updateSettings({
            temperatureUnit: event.currentTarget.value as WeatherSettings["temperatureUnit"],
          })}
      >
        <option value="celsius">{$t("celsius")}</option>
        <option value="fahrenheit">{$t("fahrenheit")}</option>
      </select>
    </label>
    <label class="weather-setting">
      <span>{$t("wind_unit")}</span>
      <select
        data-skin="default"
        value={weatherModule.settings.windSpeedUnit}
        onchange={(event) =>
          updateSettings({
            windSpeedUnit: event.currentTarget.value as WeatherSettings["windSpeedUnit"],
          })}
      >
        <option value="kmh">{$t("kilometers_per_hour")}</option>
        <option value="mph">{$t("miles_per_hour")}</option>
      </select>
    </label>
    <label class="weather-setting">
      <span>{$t("precipitation_unit")}</span>
      <select
        data-skin="default"
        value={weatherModule.settings.precipitationUnit}
        onchange={(event) =>
          updateSettings({
            precipitationUnit: event.currentTarget.value as WeatherSettings["precipitationUnit"],
          })}
      >
        <option value="mm">{$t("millimeters")}</option>
        <option value="inch">{$t("inches")}</option>
      </select>
    </label>
    <label class="weather-setting">
      <span>{$t("pressure_unit")}</span>
      <select
        data-skin="default"
        value={weatherModule.settings.pressureUnit}
        onchange={(event) =>
          updateSettings({
            pressureUnit: event.currentTarget.value as WeatherSettings["pressureUnit"],
          })}
      >
        <option value="inhg">inHg</option>
        <option value="hpa">hPa</option>
      </select>
    </label>
    </div>
    {/if}
  </section>
  {#if settingsError}
    <div class="weather-error" role="alert">{settingsError}</div>
  {/if}
</div>
