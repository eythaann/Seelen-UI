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
    { label: 'Português', value: 'pt' }, // Portuguese
    { label: 'Русский', value: 'ru' }, // Russian
    { label: 'हिन्दी', value: 'hi' }, // Hindi
    { label: '日本語', value: 'ja' }, // Japanese
    { label: 'Italiano', value: 'it' }, // Italian
    { label: 'Nederlands', value: 'nl' }, // Dutch
    { label: 'Türkçe', value: 'tr' }, // Turkish
    { label: 'Polski', value: 'pl' }, // Polish
    { label: 'Українська', value: 'uk' }, // Ukrainian
    { label: 'Ελληνικά', value: 'el' }, // Greek
    { label: 'עברית', value: 'he' }, // Hebrew
    { label: 'Svenska', value: 'sv' }, // Swedish
    { label: 'Norsk', value: 'no' }, // Norwegian
    { label: 'Suomi', value: 'fi' }, // Finnish
    { label: 'Dansk', value: 'da' }, // Danish
    { label: 'Magyar', value: 'hu' }, // Hungarian
    { label: 'Română', value: 'ro' }, // Romanian
    { label: 'Čeština', value: 'cs' }, // Czech
    { label: 'Slovenský', value: 'sk' }, // Slovak
    { label: 'Hrvatski', value: 'hr' }, // Croatian
    { label: 'Български', value: 'bg' }, // Bulgarian
    { label: 'Lietuvių', value: 'lt' }, // Lithuanian
    { label: 'Latviešu', value: 'lv' }, // Latvian
    { label: 'Eesti', value: 'et' }, // Estonian
    { label: 'Filipino', value: 'tl' }, // Filipino
    { label: 'Tiếng Việt', value: 'vi' }, // Vietnamese
    { label: 'ไทย', value: 'th' }, // Thai
    { label: 'Indonesia', value: 'id' }, // Indonesian
    { label: 'Malay', value: 'ms' }, // Malay
    { label: 'Català', value: 'ca' }, // Catalan
  ] as const),
].sort((a, b) => a.label.localeCompare(b.label));
