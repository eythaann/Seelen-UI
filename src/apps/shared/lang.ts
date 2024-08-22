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
    { label: 'Afrikaans', value: 'af' }, // Afrikaans
    { label: 'বাংলা', value: 'bn' }, // Bengali
    { label: 'فارسی', value: 'fa' }, // Farsi
    { label: 'ਪੰਜਾਬੀ', value: 'pa' }, // Punjabi
    { label: 'Kiswahili', value: 'sw' }, // Swahili
    { label: 'தமிழ்', value: 'ta' }, // Tamil
    { label: 'اردو', value: 'ur' }, // Urdu
    { label: 'Cymraeg', value: 'cy' }, // Welsh
    { label: 'አማርኛ', value: 'am' }, // Amharic
    { label: 'Հայերեն', value: 'hy' }, // Armenian
    { label: 'Azərbaycan', value: 'az' }, // Azerbaijani
    { label: 'Euskara', value: 'eu' }, // Basque
    { label: 'Bosanski', value: 'bs' }, // Bosnian
    { label: 'ქართული', value: 'ka' }, // Georgian
    { label: 'ગુજરાતી', value: 'gu' }, // Gujarati
    { label: 'Íslenska', value: 'is' }, // Icelandic
    { label: 'ភាសាខ្មែរ', value: 'km' }, // Khmer
    { label: 'Kurdî', value: 'ku' }, // Kurdish
    { label: 'ລາວ', value: 'lo' }, // Lao
    { label: 'Lëtzebuergesch', value: 'lb' }, // Luxembourgish
    { label: 'Македонски', value: 'mk' }, // Macedonian
    { label: 'Malti', value: 'mt' }, // Maltese
    { label: 'Монгол', value: 'mn' }, // Mongolian
    { label: 'नेपाली', value: 'ne' }, // Nepali
    { label: 'پښتو', value: 'ps' }, // Pashto
    { label: 'Српски', value: 'sr' }, // Serbian
    { label: 'සිංහල', value: 'si' }, // Sinhala
    { label: 'Soomaali', value: 'so' }, // Somali
    { label: 'Тоҷикӣ', value: 'tg' }, // Tajik
    { label: 'తెలుగు', value: 'te' }, // Telugu
    { label: 'Oʻzbek', value: 'uz' }, // Uzbek
    { label: 'Yorùbá', value: 'yo' }, // Yoruba
    { label: 'isiZulu', value: 'zu' }, // Zulu
  ] as const),
].sort((a, b) => a.label.localeCompare(b.label));
