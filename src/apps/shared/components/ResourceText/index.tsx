import { ResourceText } from '@seelen-ui/lib/types';
import { useTranslation } from 'react-i18next';

interface Props {
  text: ResourceText;
}

export function ResourceText({ text }: Props) {
  const {
    i18n: { language },
  } = useTranslation();

  if (typeof text === 'string') {
    return text;
  }

  return text[language] || text['en'] || null;
}
