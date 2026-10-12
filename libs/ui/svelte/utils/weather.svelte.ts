import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";

export type WeatherSettings = {
  location: string;
  useSystemLocation: boolean;
  locale: string;
  temperatureUnit: "celsius" | "fahrenheit";
  windSpeedUnit: "kmh" | "mph";
  precipitationUnit: "mm" | "inch";
  pressureUnit: "inhg" | "hpa";
  showLocation: boolean;
  showTemperature: boolean;
  showCondition: boolean;
  showRainChance: boolean;
  showPrecipitation: boolean;
  showWindSpeed: boolean;
  showWindGusts: boolean;
  showWindDirection: boolean;
  showHumidity: boolean;
  showCloudCover: boolean;
  showPressure: boolean;
  showUvIndex: boolean;
  showSunriseSunset: boolean;
  showMoonPhase: boolean;
};

export type WeatherHour = {
  time: string;
  condition: string;
  temperature: number;
  rainChance: number;
  windSpeed: number;
};

export type WeatherSnapshot = {
  location: string;
  moonPhase: { name: string; illumination: number };
  condition: string;
  temperature: number;
  feelsLike: number;
  humidity: number;
  precipitation: number;
  rainChance: number;
  windSpeed: number;
  windGusts: number;
  windDirection: number;
  cloudCover: number;
  pressure: number;
  sunrise: string;
  sunset: string;
  high: number;
  low: number;
  uvIndex: number;
  nextHours: WeatherHour[];
  updatedAt: number;
};

const REFRESH_INTERVAL = 15 * 60 * 1000;

const DEFAULT_SETTINGS: WeatherSettings = {
  location: "",
  useSystemLocation: true,
  locale: "system",
  temperatureUnit: "fahrenheit",
  windSpeedUnit: "mph",
  precipitationUnit: "inch",
  pressureUnit: "inhg",
  showLocation: false,
  showTemperature: true,
  showCondition: false,
  showRainChance: true,
  showPrecipitation: false,
  showWindSpeed: false,
  showWindGusts: false,
  showWindDirection: false,
  showHumidity: false,
  showCloudCover: false,
  showPressure: false,
  showUvIndex: false,
  showSunriseSunset: false,
  showMoonPhase: false,
};

function parseSettings(stored: string): WeatherSettings | null {
  if (!stored) return null;
  try {
    const saved = JSON.parse(stored) as Partial<WeatherSettings>;
    return {
      ...DEFAULT_SETTINGS,
      ...saved,
      useSystemLocation: saved.useSystemLocation ?? !saved.location,
    };
  } catch (error) {
    console.error("Could not load weather settings:", error);
    return null;
  }
}

function weatherCondition(code: number): string {
  if (code === 0) return "clear";
  if (code === 1) return "mainly_clear";
  if (code === 2) return "partly_cloudy";
  if (code === 3) return "overcast";
  if (code === 45 || code === 48) return "fog";
  if (code >= 51 && code <= 55) return "drizzle";
  if (code >= 56 && code <= 57) return "freezing_drizzle";
  if (code >= 61 && code <= 65) return "rain";
  if (code >= 66 && code <= 67) return "freezing_rain";
  if (code >= 71 && code <= 75) return "snow";
  if (code === 77) return "snow_grains";
  if (code >= 80 && code <= 82) return "rain_showers";
  if (code >= 85 && code <= 86) return "snow_showers";
  if (code >= 95) return "thunderstorm";
  return "unknown";
}

function calculateMoonPhase(timestamp: number): WeatherSnapshot["moonPhase"] {
  const lunarMonthDays = 29.530588853;
  const knownNewMoon = Date.UTC(2000, 0, 6, 18, 14);
  const age = ((timestamp - knownNewMoon) / 86_400_000 % lunarMonthDays + lunarMonthDays) %
    lunarMonthDays;
  const fraction = age / lunarMonthDays;
  const phases = [
    "new",
    "waxing_crescent",
    "first_quarter",
    "waxing_gibbous",
    "full",
    "waning_gibbous",
    "last_quarter",
    "waning_crescent",
  ];
  const index = Math.floor(fraction * phases.length + 0.5) % phases.length;
  const name = phases[index];
  if (!name) throw new Error("Moon phase calculation returned an invalid index");

  return {
    name,
    illumination: Math.round(((1 - Math.cos(2 * Math.PI * fraction)) / 2) * 100),
  };
}

function getCurrentPosition(signal: AbortSignal): Promise<GeolocationPosition> {
  if (!navigator.geolocation) {
    return Promise.reject(new Error("location.unsupported"));
  }

  return new Promise((resolve, reject) => {
    if (signal.aborted) {
      reject(new DOMException("Location lookup aborted", "AbortError"));
      return;
    }

    let settled = false;
    const cleanup = () => signal.removeEventListener("abort", onAbort);
    const onAbort = () => {
      settled = true;
      cleanup();
      reject(new DOMException("Location lookup aborted", "AbortError"));
    };
    signal.addEventListener("abort", onAbort, { once: true });

    navigator.geolocation.getCurrentPosition(
      (position) => {
        if (settled) return;
        settled = true;
        cleanup();
        resolve(position);
      },
      (error) => {
        if (settled) return;
        settled = true;
        cleanup();
        const key = error.code === error.PERMISSION_DENIED
          ? "location.permission_denied"
          : error.code === error.TIMEOUT
          ? "location.timeout"
          : "location.unavailable";
        reject(new Error(key));
      },
      { enableHighAccuracy: false, maximumAge: REFRESH_INTERVAL, timeout: 15_000 },
    );
  });
}

class WeatherModule {
  private settingsState = $state<WeatherSettings>({ ...DEFAULT_SETTINGS });
  private storedSettings = "";
  private settingsLoaded = false;
  private snapshotState = $state<WeatherSnapshot | null>(null);
  private loadingState = $state(false);
  private errorState = $state("");
  private consumers = 0;
  private timer: ReturnType<typeof setInterval> | null = null;
  private controller: AbortController | null = null;

  get settings(): WeatherSettings {
    return this.settingsState;
  }

  get snapshot(): WeatherSnapshot | null {
    return this.snapshotState;
  }

  get loading(): boolean {
    return this.loadingState;
  }

  get error(): string {
    return this.errorState;
  }

  private applyStored(stored: string): boolean {
    if (stored === this.storedSettings) return false;
    const parsed = parseSettings(stored);
    if (!parsed) return false;
    this.storedSettings = stored;
    this.settingsState = parsed;
    return true;
  }

  private async loadSettings(): Promise<void> {
    if (this.settingsLoaded) return;
    this.settingsLoaded = true;
    // Registered before the initial read so a change event is never lost
    await subscribe(SeelenEvent.StateSettingsChanged, ({ payload }) => {
      if (this.applyStored(payload.weatherSettings) && this.consumers > 0) {
        void this.refresh();
      }
    });
    const settings = await invoke(SeelenCommand.StateGetSettings, { path: null });
    this.applyStored(settings.weatherSettings);
  }

  activate(): () => void {
    this.consumers += 1;
    if (this.consumers === 1) {
      void this.loadSettings().catch((error) => console.error("Could not load weather settings:", error))
        .finally(() => {
          if (this.consumers > 0) void this.refresh();
        });
      this.timer = setInterval(() => void this.refresh(), REFRESH_INTERVAL);
    }

    let active = true;
    return () => {
      if (!active) return;
      active = false;
      this.consumers = Math.max(0, this.consumers - 1);
      if (this.consumers !== 0) return;
      if (this.timer) clearInterval(this.timer);
      this.timer = null;
      this.controller?.abort();
      this.controller = null;
      this.loadingState = false;
    };
  }

  async updateSettings(changes: Partial<WeatherSettings>): Promise<void> {
    const next = { ...this.settingsState, ...changes };
    try {
      const serialized = JSON.stringify(next);
      await invoke(SeelenCommand.SetWeatherSettings, { value: serialized });
      this.storedSettings = serialized;
      this.settingsState = next;
      this.errorState = "";
      if (this.consumers > 0) await this.refresh();
    } catch (error) {
      this.errorState = String(error);
      throw error;
    }
  }

  async refresh(): Promise<void> {
    const settings = this.settingsState;
    const location = settings.location.trim();
    if (!settings.useSystemLocation && !location) {
      this.controller?.abort();
      this.controller = null;
      this.snapshotState = null;
      this.loadingState = false;
      this.errorState = "";
      return;
    }

    this.controller?.abort();
    const controller = new AbortController();
    this.controller = controller;
    this.loadingState = true;
    this.errorState = "";
    try {
      let latitude: number;
      let longitude: number;
      let locationName = "";
      if (settings.useSystemLocation) {
        const position = await getCurrentPosition(controller.signal);
        latitude = position.coords.latitude;
        longitude = position.coords.longitude;
      } else {
        const geocodeUrl = new URL("https://geocoding-api.open-meteo.com/v1/search");
        geocodeUrl.search = new URLSearchParams({
          name: location,
          count: "1",
          language: "en",
          format: "json",
        }).toString();
        const geocodeResponse = await fetch(geocodeUrl, { signal: controller.signal });
        if (!geocodeResponse.ok) {
          throw new Error(`Location lookup failed (${geocodeResponse.status})`);
        }
        const geocode = await geocodeResponse.json();
        const place = geocode.results?.[0];
        if (!place) throw new Error(`No matching location found for "${location}"`);
        latitude = place.latitude;
        longitude = place.longitude;
        locationName = [place.name, place.admin1, place.country].filter(Boolean).join(", ");
      }

      const forecastUrl = new URL("https://api.open-meteo.com/v1/forecast");
      forecastUrl.search = new URLSearchParams({
        latitude: String(latitude),
        longitude: String(longitude),
        temperature_unit: settings.temperatureUnit,
        wind_speed_unit: settings.windSpeedUnit,
        precipitation_unit: settings.precipitationUnit,
        current:
          "temperature_2m,relative_humidity_2m,apparent_temperature,precipitation,weather_code,cloud_cover,pressure_msl,wind_speed_10m,wind_direction_10m,wind_gusts_10m",
        hourly: "temperature_2m,weather_code,precipitation_probability,wind_speed_10m",
        forecast_hours: "6",
        daily:
          "precipitation_probability_max,sunrise,sunset,temperature_2m_max,temperature_2m_min,uv_index_max",
        timezone: "auto",
        forecast_days: "1",
      }).toString();
      const forecastResponse = await fetch(forecastUrl, { signal: controller.signal });
      if (!forecastResponse.ok) throw new Error(`Weather lookup failed (${forecastResponse.status})`);
      const forecast = await forecastResponse.json();
      const current = forecast.current;
      const currentHour = forecast.hourly.time.indexOf(current.time);
      const forecastStart = currentHour < 0 ? 0 : currentHour;
      if (controller.signal.aborted) return;

      this.snapshotState = {
        location: locationName,
        moonPhase: calculateMoonPhase(Date.now()),
        condition: weatherCondition(current.weather_code),
        temperature: current.temperature_2m,
        feelsLike: current.apparent_temperature,
        humidity: current.relative_humidity_2m,
        precipitation: current.precipitation,
        rainChance:
          currentHour >= 0
            ? forecast.hourly.precipitation_probability[currentHour]
            : forecast.daily.precipitation_probability_max[0],
        windSpeed: current.wind_speed_10m,
        windGusts: current.wind_gusts_10m,
        windDirection: current.wind_direction_10m,
        cloudCover: current.cloud_cover,
        pressure: settings.pressureUnit === "inhg"
          ? Math.round(current.pressure_msl * 0.02953 * 100) / 100
          : Math.round(current.pressure_msl),
        sunrise: forecast.daily.sunrise[0],
        sunset: forecast.daily.sunset[0],
        high: forecast.daily.temperature_2m_max[0],
        low: forecast.daily.temperature_2m_min[0],
        uvIndex: forecast.daily.uv_index_max[0],
        nextHours: forecast.hourly.time
          .slice(forecastStart, forecastStart + 6)
          .map((time: string, index: number) => {
            const hour = forecastStart + index;
            return {
              time,
              condition: weatherCondition(forecast.hourly.weather_code[hour]),
              temperature: forecast.hourly.temperature_2m[hour],
              rainChance: forecast.hourly.precipitation_probability[hour],
              windSpeed: forecast.hourly.wind_speed_10m[hour],
            };
          }),
        updatedAt: Date.now(),
      };
    } catch (error) {
      if (!(error instanceof DOMException && error.name === "AbortError")) {
        this.errorState = String(error);
      }
    } finally {
      if (this.controller === controller) {
        this.controller = null;
        this.loadingState = false;
      }
    }
  }
}

export const weatherModule = new WeatherModule();
