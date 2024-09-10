import { Item } from '../item/infra';

import { DeviceTM } from '../../../shared/schemas/Placeholders';

interface Props {
  module: DeviceTM;
}

export function DeviceModule({ module }: Props) {
  return <Item extraVars={{}} module={module} />;
}
