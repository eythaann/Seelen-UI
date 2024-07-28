import { existsSync, readFileSync, writeFileSync } from 'fs';
import { translate } from 'google-translate-api-x';
import yaml from 'js-yaml';

import { LanguageList } from '../src/apps/shared/lang';

const toTranslate = LanguageList.map((lang) => lang.value).filter((lang) => lang !== 'en');

async function translateObject(base: any, lang: string, mut_obj: any) {
  await Promise.all(
    Object.entries(base).map(async ([key, value]) => {
      if (typeof value === 'object') {
        mut_obj[key] ??= {};
        await translateObject(value, lang, mut_obj[key]);
      }

      // avoid modify already translated values
      if (typeof value === 'string' && !mut_obj[key]) {
        const res = await translate(value, {
          from: 'en',
          to: lang,
        });
        mut_obj[key] = res.text;
      }
    }),
  );
}

async function completeTranslationsFor(app: string) {
  const path = `./src/apps/${app}/i18n/translations`;

  const en = yaml.load(readFileSync(`${path}/en.yml`, 'utf8'));
  for (const lang of toTranslate) {
    const filePath = `${path}/${lang}.yml`;
    console.log(filePath);

    if (!existsSync(filePath)) {
      writeFileSync(filePath, yaml.dump({}));
    }

    const trans = yaml.load(readFileSync(filePath, 'utf8'));
    await translateObject(en, lang, trans);
    writeFileSync(filePath, yaml.dump(trans));
  }
}

async function main() {
  await completeTranslationsFor('toolbar');
  await completeTranslationsFor('seelenweg');
  await completeTranslationsFor('settings');
  await completeTranslationsFor('update');
}

main().catch(console.error);
