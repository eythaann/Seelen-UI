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
      {
        type: 'select',
        key: 'pinnedItemsVisibility',
        label: 't::weg.items.pinned_visibility.label',
        default: null,
        options: [
          { value: 'Always', label: 't::weg.items.pinned_visibility.always' },
          { value: 'WhenPrimary', label: 't::weg.items.pinned_visibility.when_primary' },
        ],
      },
    ],
  },
];
