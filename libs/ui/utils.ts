export function nanosecondsToPlayingTime(nanoseconds: number): string {
  const hours = Math.floor(nanoseconds / 3_600_000_000_000);
  const minutes = Math.floor(nanoseconds / 60_000_000_000);
  const seconds = Math.floor((nanoseconds - minutes * 60_000_000_000) / 1_000_000_000);

  if (hours > 0) {
    return `${hours}:${minutes}:${seconds}`;
  } else {
    return `${minutes}:${seconds}`;
  }
}

export function brightnessIcon(brightness: number) {
  if (brightness >= 60) {
    return "TbBrightnessUp";
  }
  return brightness >= 30 ? "TbBrightnessDown" : "TbBrightnessDownFilled";
}

export function outputVolumeIcon(muted: boolean, volume: number) {
  if (muted) {
    return "IoVolumeMuteOutline";
  }

  if (volume >= 0.66) {
    return "IoVolumeHighOutline";
  }

  if (volume >= 0.33) {
    return "IoVolumeMediumOutline";
  }

  return volume === 0 ? "IoVolumeOffOutline" : "IoVolumeLowOutline";
}
