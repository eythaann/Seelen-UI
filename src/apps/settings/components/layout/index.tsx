import { Outlet } from 'react-router';

import { Header } from '../header';
import { Navigation } from '../navigation';
import cs from './index.module.css';

export function Layout() {
  return <div className={cs.layout}>
    <Navigation />
    <Header />
    <div className={cs.content}>
      <Outlet />
    </div>
  </div>;
}