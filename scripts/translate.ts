import { existsSync, readFileSync, writeFileSync } from 'fs';
import { translate } from 'google-translate-api-x';
import yaml from 'js-yaml';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

import { LanguageList } from '../src/apps/shared/lang';

const toTranslate = LanguageList.map((lang) => lang.value).filter((lang) => lang !== 'en');

async function translateObject(base: any, lang: string, mut_obj: any) {
  await Promise.all(
    Object.entries(base).map(async ([key, value]) => {
      if (typeof value === 'object') {
        mut_obj[key] ??= {};
        await translateObject(value, lang, mut_obj[key]);
      }

      // avoid modifying already translated values
      if (typeof value === 'string' && !mut_obj[key]) {
        const res = await translate(value, {
          from: 'en',
          to: lang,
          forceTo: true,
        });
        mut_obj[key] = res.text;
      }
    }),
  );
}

async function completeTranslationsFor(
  app: string,
  keysToUpdate: Set<string>,
  deleteKeys: Set<string>,
) {
  const path = `./src/apps/${app}/i18n/translations`;

  const en = yaml.load(readFileSync(`${path}/en.yml`, 'utf8'));
  deleteKeysDeep(en, Array.from(deleteKeys));
  writeFileSync(`${path}/en.yml`, yaml.dump(en));

  for (const lang of toTranslate) {
    const filePath = `${path}/${lang}.yml`;
    console.log(`Processing: ${filePath}`);

    let trans: any = {};
    if (existsSync(filePath)) {
      trans = yaml.load(readFileSync(filePath, 'utf8'));
    }

    deleteKeysDeep(trans, Array.from(deleteKeys));
    deleteKeysDeep(trans, Array.from(keysToUpdate));
    await translateObject(en, lang, trans);

    writeFileSync(filePath, yaml.dump(trans));
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

async function main() {
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

  await completeTranslationsFor('toolbar', keysToUpdate, deleteKeys);
  await completeTranslationsFor('seelenweg', keysToUpdate, deleteKeys);
  await completeTranslationsFor('settings', keysToUpdate, deleteKeys);
  await completeTranslationsFor('update', keysToUpdate, deleteKeys);
  await completeTranslationsFor('seelen_rofi', keysToUpdate, deleteKeys);
}

main().catch(console.error);
