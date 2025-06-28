import { ResourceText as IResourceText } from '@seelen-ui/lib/types';
import { useTranslation } from 'react-i18next';

interface Props {
  text?: IResourceText;
}

export function ResourceText({ text }: Props) {
  const {
    i18n: { language },
  } = useTranslation();

  if (!text) {
    return <span>null!?</span>;
  }

  if (typeof text === 'string') {
    return <span>{text}</span>;
  }

  return <span>{text[language] || text['en'] || 'null!?'}</span>;
}
