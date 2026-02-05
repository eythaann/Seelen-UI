import { Suspense } from "preact/compat";
import { InnerItem, type InnerItemProps } from "./Inner.tsx";

export function Item(props: InnerItemProps) {
  return (
    <Suspense fallback={null}>
      <InnerItem {...props} />
    </Suspense>
  );
}
