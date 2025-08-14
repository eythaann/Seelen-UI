import { useLayoutEffect, useRef } from 'preact/hooks';
import { Outlet, useLocation } from 'react-router';

import { Header } from '../header';
import { Navigation } from '../navigation';
import cs from './index.module.css';

export function Layout() {
  const location = useLocation();
  const contentRef = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    // Scroll to the top of the page when the route changes
    contentRef.current?.scrollTo({ top: 0, left: 0, behavior: 'instant' });
  }, [location.pathname]);

  return (
    <div className={cs.layout}>
      <Navigation />
      <Header />
      <div ref={contentRef} className={cs.content}>
        <Outlet />
      </div>
    </div>
  );
}
