import { DeviceTM } from '../../../shared/schemas/Placeholders';

import { Item } from '../item/infra';

interface Props {
  module: DeviceTM;
}

export function DeviceModule({ module }: Props) {
  return <Item extraVars={{}} module={module} />;
}
