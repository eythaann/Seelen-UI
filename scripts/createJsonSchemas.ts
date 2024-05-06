import { writeFileSync } from 'fs';
import { zodToJsonSchema } from 'zod-to-json-schema';

import { LayoutSchema } from '../src/apps/utils/schemas/Layout';
import { PlaceholderSchema } from '../src/apps/utils/schemas/Placeholders';
import { SettingsSchema } from '../src/apps/utils/schemas/Settings';
import { ThemeSchema } from '../src/apps/utils/schemas/Theme';

(async function main() {
  writeFileSync(
    'documentation/schemas/layout.schema.json',
    JSON.stringify(zodToJsonSchema(LayoutSchema), null, 2),
  );

  writeFileSync(
    'documentation/schemas/settings.schema.json',
    JSON.stringify(zodToJsonSchema(SettingsSchema), null, 2),
  );

  writeFileSync(
    'documentation/schemas/theme.schema.json',
    JSON.stringify(zodToJsonSchema(ThemeSchema), null, 2),
  );

  writeFileSync(
    'documentation/schemas/placeholder.schema.json',
    JSON.stringify(zodToJsonSchema(PlaceholderSchema), null, 2),
  );
})();
