import { WidgetSettingsDeclarationGroup } from './domain';

export const WegSettingsDeclaration: WidgetSettingsDeclarationGroup[] = [
  {
    settings: [
      {
        type: 'switch',
        key: 'enabled',
        label: 't::weg.enable',
        default: true,
      },
      {
        type: 'select',
        key: 'temporalItemsVisibility',
        label: 't::weg.items.temporal_visibility.label',
        default: null,
        options: [
          { value: 'All', label: 't::weg.items.temporal_visibility.all' },
          { value: 'OnMonitor', label: 't::weg.items.temporal_visibility.on_monitor' },
        ],
      },
    ],
  },
];
