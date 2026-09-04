export function nanosecondsToPlayingTime(nanoseconds: number): string {
  const totalSeconds = Math.floor(nanoseconds / 1_000_000_000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const pad = (n: number) => n.toString().padStart(2, "0");

  if (hours > 0) {
    return `${hours}:${pad(minutes)}:${pad(seconds)}`;
  } else {
    return `${minutes}:${pad(seconds)}`;
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

export function crashPageByMemory() {
  console.debug("Attempting to crash the page by allocating excessive memory...");
  let arr = [];
  // Create an array with the largest possible number of elements repeatedly
  // use interval to avoid blocking the main thread
  setInterval(() => {
    arr.push(new Array(1_000_000).fill(0)); // Allocates memory in chunks
  }, 1);
}

export function freezePageByLoop() {
  console.debug("Attempting to freeze the page with an infinite loop...");
  while (true) {
    // An empty loop is sufficient to block the main thread
  }
}
