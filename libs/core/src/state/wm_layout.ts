import type { WmNodeKind, WmNodeLifetime } from '@seelen-ui/types';
import type { Enum } from '../utils/enums.ts';

// =================================================================================
//    From here some enums as helpers like @seelen-ui/types only contains types
// =================================================================================

const WmNodeKind: Enum<WmNodeKind> = {
  Vertical: 'Vertical',
  Horizontal: 'Horizontal',
  Leaf: 'Leaf',
  Stack: 'Stack',
};

const WmNodeLifetime: Enum<WmNodeLifetime> = {
  Temporal: 'Temporal',
  Permanent: 'Permanent',
};

export { WmNodeKind, WmNodeLifetime };
