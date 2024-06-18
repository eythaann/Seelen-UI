import { Icon } from '../../../../shared/components/Icon';
import {
  ApplicationIdentifier,
  IdWithIdentifier,
  MatchingStrategy,
} from '../../../../shared/schemas/AppsConfigurations';
import { SettingsGroup, SettingsOption } from '../../../components/SettingsBox';
import { Button, Input, Select, Switch } from 'antd';

import { OptionsFromEnum } from '../../shared/utils/app';

import cs from './Identifier.module.css';

interface Props {
  identifier: IdWithIdentifier;
  onChange: (id: IdWithIdentifier) => void;
  onRemove?: () => void;
}

export function Identifier({ identifier, onChange, onRemove }: Props) {
  const { id, kind, matchingStrategy } = identifier;

  const onChangeId = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange({ ...identifier, id: e.target.value });
  };

  const onSelectKind = (value: ApplicationIdentifier) => {
    onChange({ ...identifier, kind: value });
  };

  const onSelectMatchingStrategy = (value: MatchingStrategy) => {
    onChange({ ...identifier, matchingStrategy: value });
  };

  const onChangeNegation = (value: boolean) => {
    onChange({ ...identifier, negation: value });
  };

  const onChangeAndItem = (idx: number, value: IdWithIdentifier) => {
    onChange({ ...identifier, and: identifier.and.map((id, i) => (i === idx ? value : id)) });
  };

  const onChangeOrItem = (idx: number, value: IdWithIdentifier) => {
    onChange({ ...identifier, or: identifier.or.map((id, i) => (i === idx ? value : id)) });
  };

  const onRemoveAndItem = (idx: number) => {
    onChange({ ...identifier, and: identifier.and.filter((_, i) => i !== idx) });
  };

  const onRemoveOrItem = (idx: number) => {
    onChange({ ...identifier, or: identifier.or.filter((_, i) => i !== idx) });
  };

  const onAddAndItem = () => {
    onChange({ ...identifier, and: [IdWithIdentifier.default(), ...identifier.and] });
  };

  const onAddOrItem = () => {
    onChange({ ...identifier, or: [IdWithIdentifier.default(), ...identifier.or] });
  };

  return (
    <SettingsGroup>
      <div>
        {onRemove && (
          <SettingsOption>
            <span>Remove</span>
            <Button type="text" danger onClick={onRemove} className={cs.removeButton}>
              <Icon iconName="IoTrash" />
            </Button>
          </SettingsOption>
        )}
        <SettingsOption>
          <span>Identifier</span>
          <Input value={id} onChange={onChangeId} />
        </SettingsOption>
        <SettingsOption>
          <span>Identify By</span>
          <Select
            value={kind}
            options={OptionsFromEnum(ApplicationIdentifier)}
            onSelect={onSelectKind}
          />
        </SettingsOption>
        <SettingsOption>
          <span>Matching Strategy</span>
          <Select
            value={matchingStrategy}
            options={OptionsFromEnum(MatchingStrategy)}
            onSelect={onSelectMatchingStrategy}
          />
        </SettingsOption>
        <SettingsOption>
          <span>Negate Matching</span>
          <Switch value={identifier.negation} onChange={onChangeNegation} />
        </SettingsOption>

        <SettingsOption>
          <b>AND</b>
          <Button type="dashed" onClick={onAddAndItem}>
            Add Block
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
          <b>OR</b>
          <Button type="dashed" onClick={onAddOrItem}>
            Add Block
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
