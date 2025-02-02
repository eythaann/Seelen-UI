import { useTranslation } from 'react-i18next';

export function EmptyList() {
  const { t } = useTranslation();
  return <div className="userhome-empty-list">{t('userhome.empty_list')}</div>;
}
