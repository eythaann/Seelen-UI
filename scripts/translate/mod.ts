import { SupportedLanguages } from '@seelen-ui/lib';
import * as deepl from 'deepl-node';
import { existsSync, readFileSync, writeFileSync } from 'fs';
import * as GoogleTranslator from 'google-translate-api-x';
import yaml from 'js-yaml';
import _ from 'lodash';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

import { DeeplSupportedTargetLanguages, Translator } from './constants';

const API_KEY = process.env.DEEPL_API_KEY;

if (!API_KEY) {
  console.error('Missing DEEPL_API_KEY');
  process.exit(1);
}

const targets = SupportedLanguages.filter((lang) => lang.value !== 'en');

const DeeplTranslator = new deepl.Translator(API_KEY);

const argv = await yargs(hideBin(process.argv)).option('recreate', {
  type: 'array',
  description: 'Path of object to recreate translations for. (e.g. `obj.prop.deep.key`)',
  alias: 'r',
  coerce: (arg) => (Array.isArray(arg) ? arg.map(String) : [String(arg)]),
}).argv;

const keysToRecreate = new Set(argv.recreate || []);

function deepObjectSize(obj: any, size = 0) {
  for (const key in obj) {
    if (typeof obj[key] === 'object') {
      size = deepObjectSize(obj[key], size);
    } else {
      size += 1;
    }
  }
  return size;
}

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

async function translateObject(base: any, lang: string, mut_obj: any, translator: Translator) {
  for (const [key, value] of Object.entries(base)) {
    if (typeof value === 'object') {
      mut_obj[key] ??= {};
      await translateObject(value, lang, mut_obj[key], translator);
    }
    // avoid modifying already translated values
    if (typeof value === 'string' && !mut_obj[key]) {
      if (translator === Translator.DeepL) {
        const res = await DeeplTranslator.translateText(
          value,
          'en',
          lang as deepl.TargetLanguageCode,
        );
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
  // remove obsolete keys
  for (const key in mut_obj) {
    if (!base[key]) {
      delete mut_obj[key];
    }
  }
}

function deleteKeysDeep(obj: any, keys: string[]) {
  const deletePath = (obj: any, path: string[]) => {
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
  };

  for (const key of keys) {
    deletePath(obj, key.split('.'));
  }
}

async function completeTranslationsFor(localesDir: string) {
  const enPath = `${localesDir}/en.yml`;
  const en = deepSortObject(yaml.load(readFileSync(enPath, 'utf8')) as object);
  writeFileSync(enPath, yaml.dump(en)); // overwrite sorted

  console.log(`* ${enPath} (total: ${deepObjectSize(en)} messages)`);

  for (const item of targets) {
    const filePath = `${localesDir}/${item.value}.yml`;
    const translator = DeeplSupportedTargetLanguages.includes(
      item.value as deepl.TargetLanguageCode,
    )
      ? Translator.DeepL
      : Translator.Google;

    console.log(`  - ${filePath} (${item.enLabel}) - ${translator}`);

    let translation: any = {};
    if (existsSync(filePath)) {
      translation = yaml.load(readFileSync(filePath, 'utf8'));
    }

    deleteKeysDeep(translation, Array.from(keysToRecreate));
    await translateObject(en, item.value, translation, translator);

    writeFileSync(filePath, yaml.dump(deepSortObject(translation)));
  }
  console.log(); // newline on finish
}

await completeTranslationsFor('src/apps/toolbar/i18n/translations');
await completeTranslationsFor('src/apps/seelenweg/i18n/translations');
await completeTranslationsFor('src/apps/settings/i18n/translations');
await completeTranslationsFor('src/apps/seelen_rofi/i18n/translations');

await completeTranslationsFor('src/background/i18n');
