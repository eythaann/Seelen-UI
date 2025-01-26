import { TargetLanguageCode } from 'deepl-node';

export const DeeplSupportedTargetLanguages = [
  'ar',
  'bg',
  'cs',
  'da',
  'de',
  'el',
  'es',
  'et',
  'fi',
  'fr',
  'hu',
  'id',
  'it',
  'ja',
  'ko',
  'lt',
  'lv',
  'nb',
  'nl',
  'pl',
  'ro',
  'ru',
  'sk',
  'sl',
  'sv',
  'tr',
  'uk',
  'zh',
  'en-GB',
  'en-US',
  'pt-BR',
  'pt-PT',
] as const;

type OwnList = (typeof DeeplSupportedTargetLanguages)[number];
type areAllDeeplTargetLanguagesOnTheList = [TargetLanguageCode] extends [OwnList]
  ? [OwnList] extends [TargetLanguageCode]
    ? true
    : false
  : false;

const _assert: areAllDeeplTargetLanguagesOnTheList = true;
