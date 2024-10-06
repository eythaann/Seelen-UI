import { Skeleton } from 'antd';
import { useState } from 'react';

import cs from './MiniStore.module.css';

export function ProductSkeleton() {
  return (
    <div className={cs.product}>
      <Skeleton.Image active />
      <Skeleton active paragraph={false} />
    </div>
  );
}

export function MiniStore() {
  const [products, _setProducts] = useState<any[]>([]);

  return (
    <>
      <h1 className={cs.title}>New Resources</h1>
      <div className={cs.miniStore}>
        {products.length === 0 &&
          Array.from({ length: 10 }).map((_, i) => <ProductSkeleton key={i} />)}
      </div>
    </>
  );
}
