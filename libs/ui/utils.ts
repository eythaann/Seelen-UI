/**
 * Deterministic UUID-shaped id derived from `key` (an "idempotency key"
 * pattern: same logical entity -> same id, regardless of who computes it or
 * when).
 *
 * Use this whenever multiple independent producers (e.g. one webview per
 * monitor, watching the same shared state) may each need to mint an id for
 * "the same" entity without coordinating with each other first. Deriving the
 * id from the entity's own identity instead of randomizing it means every
 * producer converges on the same id up front, so downstream id-based dedup
 * collapses concurrent creations for free instead of surfacing a visible
 * duplicate that only gets reconciled away later.
 *
 * Algorithm: cyrb128, a fast 128-bit (4x uint32) non-cryptographic string
 * hash (public domain, by bryc). It is NOT collision-resistant against an
 * adversary who can choose inputs -- it has no formal collision-resistance
 * proof and is not meant to defend against intentionally crafted collisions.
 * Only use it for keys that come from trusted/internal sources (OS state,
 * our own data), never for untrusted/attacker-controlled input. What it does
 * give us is a large output space with good avalanche behavior, so
 * *unrelated* keys don't accidentally collide: for the birthday bound to
 * give even a 1e-6 chance of any collision at 128 bits of output, you'd need
 * on the order of 10^13 distinct concurrent keys. If you need real
 * collision-resistance against adversarial input, use
 * `crypto.subtle.digest` instead -- note that it's Promise-based, so it
 * won't drop into a synchronous call site the way this does.
 *
 * The four 32-bit words are formatted into standard 8-4-4-4-12 UUID layout
 * (with a forced version/variant nibble) purely so the value round-trips
 * through APIs/fields typed as UUID (e.g. Rust `uuid::Uuid`) -- the value
 * itself is a content hash, not a "real" random/time-based UUID.
 */
export function stableUUID(key: string): string {
  // cyrb128: https://github.com/bryc/code/blob/master/jshash/PRNGs.md
  let h1 = 1779033703, h2 = 3144134277, h3 = 1013904242, h4 = 2773480762;
  for (let i = 0; i < key.length; i++) {
    const k = key.charCodeAt(i);
    h1 = h2 ^ Math.imul(h1 ^ k, 597399067);
    h2 = h3 ^ Math.imul(h2 ^ k, 2869860233);
    h3 = h4 ^ Math.imul(h3 ^ k, 951274213);
    h4 = h1 ^ Math.imul(h4 ^ k, 2716044179);
  }
  // final mix (avalanche): spreads bit changes from any single input
  // character across all 4 output words, so similar keys (e.g. paths
  // differing by one letter) don't produce similar/adjacent hashes.
  h1 = Math.imul(h3 ^ (h1 >>> 18), 597399067) >>> 0;
  h2 = Math.imul(h4 ^ (h2 >>> 22), 2869860233) >>> 0;
  h3 = Math.imul(h1 ^ (h3 >>> 17), 951274213) >>> 0;
  h4 = Math.imul(h2 ^ (h4 >>> 19), 2716044179) >>> 0;
  const hex = [h1, h2, h3, h4].map((n) => n.toString(16).padStart(8, "0")).join("");
  // Standard UUID layout (8-4-4-4-12). The '4' and '89ab' nibbles are forced
  // version/variant markers only so the string is a well-formed UUID for
  // deserialization -- they don't carry real UUIDv4 semantics since this
  // isn't random.
  return [
    hex.slice(0, 8),
    hex.slice(8, 12),
    `4${hex.slice(13, 16)}`,
    `${"89ab"[h4 & 3]}${hex.slice(17, 20)}`,
    hex.slice(20, 32),
  ].join("-");
}

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
