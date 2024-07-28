export type Lang = (typeof LanguageList)[number]['value'];

export const LanguageList = [
  ...([
    { label: 'Deutsch', value: 'de' }, // German
    { label: 'English', value: 'en' }, // English
    { label: 'Español', value: 'es' }, // Spanish
    { label: '한국어', value: 'ko' }, // Korean
    { label: '中文', value: 'zh' }, // Chinese
    { label: 'Français', value: 'fr' }, // French
    { label: 'العربية', value: 'ar' }, // Arabic
  ] as const),
].sort((a, b) => a.label.localeCompare(b.label));
