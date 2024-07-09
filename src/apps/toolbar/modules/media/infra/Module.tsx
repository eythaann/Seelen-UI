import { MediaTM } from '../../../../shared/schemas/Placeholders';
import { WithMediaControls } from './MediaControls';

import { Item } from '../../item/infra';

interface Props {
  module: MediaTM;
}

function MediaModuleItem({ module }: Props) {
  return (
    <Item
      extraVars={{}}
      module={{
        ...module,
        onClick: module.withMediaControls ? 'nothing' : module.onClick,
      }}
    />
  );
}

export function MediaModule({ module }: Props) {
  return module.withMediaControls ? (
    <WithMediaControls>
      <MediaModuleItem module={module} />
    </WithMediaControls>
  ) : (
    <MediaModuleItem module={module} />
  );
}
