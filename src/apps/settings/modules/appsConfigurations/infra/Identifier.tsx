import { Button, Input, Select, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { AppIdentifier, AppIdentifierType, MatchingStrategy } from 'seelen-core';

import { OptionsFromEnum } from '../../shared/utils/app';

import { Icon } from '../../../../shared/components/Icon';
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
    onChange({ ...identifier, id: e.target.value });
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
    onChange({ ...identifier, and: [AppIdentifier.create(), ...identifier.and] });
  };

  const onAddOrItem = () => {
    onChange({ ...identifier, or: [AppIdentifier.create(), ...identifier.or] });
  };

  return (
    <SettingsGroup>
      <div>
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
          <Select
            value={kind}
            options={OptionsFromEnum(AppIdentifierType)}
            onSelect={onSelectKind}
          />
        </SettingsOption>
        <SettingsOption>
          <span>{t('apps_configurations.identifier.matching_strategy')}</span>
          <Select
            value={matchingStrategy}
            options={OptionsFromEnum(MatchingStrategy)}
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
      </div>
    </SettingsGroup>
  );
}
