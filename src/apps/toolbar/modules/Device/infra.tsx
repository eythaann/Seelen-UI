import { DeviceTM } from 'seelen-core';

import { Item } from '../item/infra';

interface Props {
  module: DeviceTM;
}

export function DeviceModule({ module }: Props) {
  return <Item extraVars={{}} module={module} />;
}
