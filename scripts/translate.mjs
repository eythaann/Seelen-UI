import { existsSync, readFileSync, writeFileSync } from 'fs';
import yaml from 'js-yaml';
import translate from 'translate';

const toTranslate = [
  'es',
  'de',
  'ko',
  'zh',
  'fr',
  'ar',
];

async function translateObject(base, lang, mut_obj) {
  for (const [key, value] of Object.entries(base)) {
    if (typeof value === 'object') {
      mut_obj[key] ??= {};
      await translateObject(value, lang, mut_obj[key]);
    }

    // avoid modify already translated values
    if (typeof value === 'string' && !mut_obj[key]) {
      mut_obj[key] = await translate(value, {
        from: 'en',
        to: lang,
      });
    }
  }
}

async function completeTranslationsFor(app) {
  const path = `./src/apps/${app}/i18n/translations`;

  const en = yaml.load(readFileSync(`${path}/en.yml`, 'utf8'));

  for (const lang of toTranslate) {
    console.log(`Translating to ${lang} for ${app}.`);
    const filePath = `${path}/${lang}.yml`;

    if (!existsSync(filePath)) {
      writeFileSync(filePath, yaml.dump({}));
    }

    const trans = yaml.load(readFileSync(filePath, 'utf8'));
    await translateObject(en, lang, trans);
    writeFileSync(filePath, yaml.dump(trans));
  }
}

Promise.all([
  completeTranslationsFor('toolbar'),
  completeTranslationsFor('seelenweg'),
  completeTranslationsFor('settings'),
  completeTranslationsFor('update'),
]).catch(console.error);