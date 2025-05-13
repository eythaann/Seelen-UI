import { Translator } from '@seelen/translation-toolkit';
import { SupportedLanguages } from '@seelen-ui/lib';
import { ResourceText } from '@seelen-ui/lib/types';
import { readFileSync } from 'fs';
import { writeFile } from 'fs/promises';
import yaml from 'js-yaml';

const targetLanguages = SupportedLanguages.filter((lang) => lang.value !== 'en');

export async function completeResourceTranslations(
  resourceFile: string,
  translator: Translator<string, string>,
) {
  const strYaml = readFileSync(resourceFile, 'utf8');
  const resource: any = yaml.load(strYaml);

  resource.metadata ??= {};

  const displayName: ResourceText | undefined = resource.metadata?.displayName;
  if (displayName) {
    resource.metadata.displayName = await completeTranslationsForResourceText(
      displayName,
      translator,
    );
  }

  const description: ResourceText | undefined = resource.metadata?.description;
  if (description) {
    resource.metadata.description = await completeTranslationsForResourceText(
      description,
      translator,
    );
  }

  await writeFile(resourceFile, yaml.dump(resource));
}

async function completeTranslationsForResourceText(
  resourceText: ResourceText,
  translator: Translator<string, string>,
): Promise<ResourceText> {
  const translated: Exclude<ResourceText, string> =
    typeof resourceText === 'object' ? { ...resourceText } : {};

  if (typeof resourceText === 'string') {
    translated['en'] = resourceText;
  }

  const source = translated['en'];
  if (!source) {
    throw new Error('Missing english source text');
  }

  const untranslatedTargets = targetLanguages
    .filter((lang) => !translated[lang.value])
    .map((lang) => lang.value);

  return {
    ...translated,
    ...(await translator.translate_to_multiple(untranslatedTargets, source)),
  };
}
