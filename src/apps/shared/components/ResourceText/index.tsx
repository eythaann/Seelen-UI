import { ResourceText as IResourceText } from '@seelen-ui/lib/types';
import { useTranslation } from 'react-i18next';

interface Props {
  text?: IResourceText;
  noFallback?: boolean;
}

export function ResourceText({ text, noFallback }: Props) {
  const {
    i18n: { language },
  } = useTranslation();

  if (!text) {
    if (noFallback) {
      return null;
    }
    return <span>null!?</span>;
  }

  if (typeof text === 'string') {
    return <span>{text}</span>;
  }

  const text2 = text[language] || text['en'];
  if (!text2) {
    if (noFallback) {
      return null;
    }
    return <span>null!?</span>;
  }

  return <span>{text2}</span>;
}
