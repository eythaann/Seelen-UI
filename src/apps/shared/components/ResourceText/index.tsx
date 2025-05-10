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
    return <span>{text}</span>;
  }
  const text2 = text[language] || text['en'];
  if (!text2) {
    return null;
  }
  return <span>{text2}</span>;
}
