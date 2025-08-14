import { AppIdentifierType, MatchingStrategy } from '@seelen-ui/lib';
import { AppIdentifier } from '@seelen-ui/lib/types';
import { Icon } from '@shared/components/Icon';
import { Button, Input, Select, Switch } from 'antd';
import { cloneDeep } from 'lodash';
import { useTranslation } from 'react-i18next';

import { OptionsFromEnum } from '../../shared/utils/app';
import { defaultAppConfig } from '../app/default';

import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import cs from './Identifier.module.css';

interface Props {
  identifier: AppIdentifier;
  onChange: (id: AppIdentifier) => void;
  onRemove?: () => void;
}

export function Identifier({ identifier, onChange, onRemove }: Props) {
  const { id, kind, matchingStrategy } = identifier;

  const { t } = useTranslation();

  const onChangeId = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({ ...identifier, id: e.currentTarget.value });
  };

  const onSelectKind = (value: AppIdentifierType) => {
    onChange({ ...identifier, kind: value });
  };

  const onSelectMatchingStrategy = (value: MatchingStrategy) => {
    onChange({ ...identifier, matchingStrategy: value });
  };

  const onChangeNegation = (value: boolean) => {
    onChange({ ...identifier, negation: value });
  };

  const onChangeAndItem = (idx: number, value: AppIdentifier) => {
    onChange({ ...identifier, and: identifier.and.map((id, i) => (i === idx ? value : id)) });
  };

  const onChangeOrItem = (idx: number, value: AppIdentifier) => {
    onChange({ ...identifier, or: identifier.or.map((id, i) => (i === idx ? value : id)) });
  };

  const onRemoveAndItem = (idx: number) => {
    onChange({ ...identifier, and: identifier.and.filter((_, i) => i !== idx) });
  };

  const onRemoveOrItem = (idx: number) => {
    onChange({ ...identifier, or: identifier.or.filter((_, i) => i !== idx) });
  };

  const onAddAndItem = () => {
    onChange({ ...identifier, and: [cloneDeep(defaultAppConfig.identifier), ...identifier.and] });
  };

  const onAddOrItem = () => {
    onChange({ ...identifier, or: [cloneDeep(defaultAppConfig.identifier), ...identifier.or] });
  };

  return (
    <SettingsGroup>
      {onRemove && (
        <SettingsOption>
          <span>{t('apps_configurations.identifier.remove')}</span>
          <Button type="text" danger onClick={onRemove} className={cs.removeButton}>
            <Icon iconName="IoTrash" />
          </Button>
        </SettingsOption>
      )}
      <SettingsOption>
        <span>{t('apps_configurations.identifier.id')}</span>
        <Input value={id} onChange={onChangeId} />
      </SettingsOption>
      <SettingsOption>
        <span>{t('apps_configurations.identifier.kind')}</span>
        <Select value={kind} options={OptionsFromEnum(t, AppIdentifierType, 'apps_configurations.identifier.type')} onSelect={onSelectKind} />
      </SettingsOption>
      <SettingsOption>
        <span>{t('apps_configurations.identifier.matching_strategy')}</span>
        <Select
          value={matchingStrategy}
          options={OptionsFromEnum(t, MatchingStrategy, 'apps_configurations.identifier.matching_strategy_option')}
          onSelect={onSelectMatchingStrategy}
        />
      </SettingsOption>
      <SettingsOption>
        <span>{t('apps_configurations.identifier.negation')}</span>
        <Switch value={identifier.negation} onChange={onChangeNegation} />
      </SettingsOption>

      <hr />

      <SettingsOption>
        <b>{t('apps_configurations.identifier.and')}</b>
        <Button type="dashed" onClick={onAddAndItem}>
          {t('apps_configurations.identifier.add_block')}
        </Button>
      </SettingsOption>
      {identifier.and.map((id, idx) => (
        <Identifier
          key={idx}
          identifier={id}
          onChange={(value) => onChangeAndItem(idx, value)}
          onRemove={() => onRemoveAndItem(idx)}
        />
      ))}

      <SettingsOption>
        <b>{t('apps_configurations.identifier.or')}</b>
        <Button type="dashed" onClick={onAddOrItem}>
          {t('apps_configurations.identifier.add_block')}
        </Button>
      </SettingsOption>
      {identifier.or.map((id, idx) => (
        <Identifier
          key={idx}
          identifier={id}
          onChange={(value) => onChangeOrItem(idx, value)}
          onRemove={() => onRemoveOrItem(idx)}
        />
      ))}
    </SettingsGroup>
  );
}
