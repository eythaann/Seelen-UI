import { useItemScope } from "../../shared/state/scope.ts";
import { InnerItem, type InnerItemProps } from "./Inner.tsx";

export function Item(props: InnerItemProps) {
  const scope = useItemScope(props.module.scopes);
  return <InnerItem {...props} extraVars={scope} />;
}
