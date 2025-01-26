import { SupportedLanguages } from '@seelen-ui/lib';
import * as deepl from 'deepl-node';
import { existsSync, readFileSync, writeFileSync } from 'fs';
import * as GoogleTranslator from 'google-translate-api-x';
import yaml from 'js-yaml';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

import { DeeplSupportedTargetLanguages } from './constants';

const API_KEY = process.env.DEEPL_API_KEY;

if (!API_KEY) {
  console.error('Missing DEEPL_API_KEY');
  process.exit(1);
}

const DeeplTranslator = new deepl.Translator(API_KEY);

const argv = await yargs(hideBin(process.argv))
  .option('delete', {
    type: 'array',
    description: 'Keys to delete from translations',
    alias: 'd',
    coerce: (arg) => (Array.isArray(arg) ? arg.map(String) : [String(arg)]),
  })
  .option('update', {
    type: 'array',
    description: 'Keys to update in translations',
    alias: 'u',
    coerce: (arg) => (Array.isArray(arg) ? arg.map(String) : [String(arg)]),
  }).argv;

const deleteKeys = new Set(argv.delete || []);
const keysToUpdate = new Set(argv.update || []);

function deepSortObject<T>(obj: T): T {
  if (Array.isArray(obj)) {
    // if it's an array, recursively sort its elements
    return obj.map(deepSortObject) as unknown as T;
  } else if (obj !== null && typeof obj === 'object') {
    // if it's an object, sort its entries
    const sortedEntries = Object.entries(obj)
      .sort(([keyA], [keyB]) => keyA.localeCompare(keyB)) // Sort keys
      .map(([key, value]) => [key, deepSortObject(value)]); // Recursively sort values

    return Object.fromEntries(sortedEntries) as T;
  }
  // if it's not an array or object, return it as is
  return obj;
}

async function translateObject(base: object, lang: string, mut_obj: any) {
  for (const [key, value] of Object.entries(base)) {
    if (typeof value === 'object') {
      mut_obj[key] ??= {};
      await translateObject(value, lang, mut_obj[key]);
    }
    // avoid modifying already translated values
    if (typeof value === 'string' && !mut_obj[key]) {
      if (DeeplSupportedTargetLanguages.includes(lang as deepl.TargetLanguageCode)) {
        const res = await DeeplTranslator.translateText(value, 'en', lang as deepl.TargetLanguageCode);
        mut_obj[key] = res.text;
      } else {
        const res = await GoogleTranslator.translate(value, {
          from: 'en',
          to: lang,
          forceTo: true,
          forceBatch: false,
        });
        mut_obj[key] = res.text;
      }
    }
  }
}

function deleteKeysDeep(obj: any, keys: string[]) {
  for (const key of keys) {
    deleteDeepKey(obj, key.split('.'));
  }
}

function deleteDeepKey(obj: any, path: string[]) {
  if (path.length === 0) {
    return;
  }

  let temp = obj;
  let finalKey = path.pop()!;

  for (const key of path) {
    if (typeof temp[key] !== 'object') {
      return;
    }
    temp = temp[key];
  }

  delete temp[finalKey];
}

const toTranslate = SupportedLanguages.map((lang) => lang.value).filter((lang) => lang !== 'en');

async function completeTranslationsFor(localesDir: string) {
  const en = deepSortObject(yaml.load(readFileSync(`${localesDir}/en.yml`, 'utf8')) as object);
  deleteKeysDeep(en, Array.from(deleteKeys));
  writeFileSync(`${localesDir}/en.yml`, yaml.dump(en));

  for (const lang of toTranslate) {
    const filePath = `${localesDir}/${lang}.yml`;
    console.log(`Processing: ${filePath}`);

    let translation: any = {};
    if (existsSync(filePath)) {
      translation = yaml.load(readFileSync(filePath, 'utf8'));
    }

    deleteKeysDeep(translation, Array.from(deleteKeys));
    deleteKeysDeep(translation, Array.from(keysToUpdate));
    await translateObject(en, lang, translation);

    writeFileSync(filePath, yaml.dump(deepSortObject(translation)));
  }
}

await completeTranslationsFor('src/apps/toolbar/i18n/translations');
await completeTranslationsFor('src/apps/seelenweg/i18n/translations');
await completeTranslationsFor('src/apps/settings/i18n/translations');
await completeTranslationsFor('src/apps/seelen_rofi/i18n/translations');

await completeTranslationsFor('src/background/i18n');
