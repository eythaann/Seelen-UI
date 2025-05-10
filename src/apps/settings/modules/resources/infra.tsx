import { ResourceKind } from '@seelen-ui/lib/types';
import { useTranslation } from 'react-i18next';
import { NavLink, Route, Routes } from 'react-router';

import cs from './infra.module.css';

import { RoutePath } from '../../components/navigation/routes';
import { ResourceKindIcon } from './common';
import { IconPacksView } from './IconPacks';
import { ThemesView } from './Themes';

const kinds: ResourceKind[] = ['Theme', 'Plugin', 'Widget', 'Wallpaper', 'IconPack', 'SoundPack'];

export function ResourcesView() {
  return (
    <Routes>
      <Route index Component={KindSelector} />
      <Route path="theme" Component={ThemesView} />
      <Route path="plugin" Component={Todo} />
      <Route path="widget" Component={Todo} />
      <Route path="wallpaper" Component={Todo} />
      <Route path="iconpack" Component={IconPacksView} />
      <Route path="soundpack" Component={Todo} />
    </Routes>
  );
}

function KindSelector() {
  const { t } = useTranslation();

  return (
    <div className={cs.kindSelector}>
      {kinds.map((kind) => (
        <NavLink key={kind} to={`${RoutePath.Resource}/${kind.toLowerCase()}`} className={cs.kind}>
          <ResourceKindIcon kind={kind} />
          <b>{t(`header.labels.${kind.toLowerCase()}`)}</b>
        </NavLink>
      ))}
    </div>
  );
}

function Todo() {
  return <div>Todo</div>;
}
