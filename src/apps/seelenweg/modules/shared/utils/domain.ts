const UWP_TARGET_SIZE_POSTFIXES = [
  '.targetsize-256_altform-lightunplated.png',
  '.targetsize-256.png',
  '.targetsize-96_altform-lightunplated.png',
  '.targetsize-96.png',
  '.targetsize-64_altform-lightunplated.png',
  '.targetsize-64.png',
  '.targetsize-48_altform-lightunplated.png',
  '.targetsize-48.png',
  '.targetsize-32_altform-lightunplated.png',
  '.targetsize-32.png',
];

const UWP_SCALE_POSTFIXES = [
  '.scale-400.png',
  '.scale-200.png',
  '.scale-150.png',
  '.scale-125.png',
  '.scale-100.png',
];

export const UWP_IMAGE_POSTFIXES = [
  ...UWP_SCALE_POSTFIXES,
  ...UWP_TARGET_SIZE_POSTFIXES,
  '.png',
];