import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { Button, Input, InputNumber, Modal, Select, Switch } from 'antd';
import React, { useState } from 'react';

import { useAppSelector } from '../../shared/app/hooks';
import { Rect } from '../../shared/app/Rect';
import { RootSelectors } from '../../shared/app/selectors';
import { OptionsFromEnum } from '../../shared/app/utils';

import { AppConfiguration, ApplicationIdentifier, ApplicationOptions, MatchingStrategy } from '../domain';

import cs from './index.module.css';

interface Props {
  idx?: number;
  open: boolean;
  onSave: (app: AppConfiguration) => void;
  onCancel: () => void;
  isNew?: boolean;
}

export const EditAppModal = ({ idx, onCancel, onSave, isNew, open }: Props) => {
  const monitors = useAppSelector(RootSelectors.monitors);
  const _app = useAppSelector((state) => {
    return idx != null && !isNew ? state.appsConfigurations[idx]! : AppConfiguration.default();
  })!;
  const [app, setApp] = useState(_app);
  const { invisibleBorders } = app;

  const onInternalSave = () => {
    onSave(app);
  };

  const updateName = (e: React.ChangeEvent<HTMLInputElement>) => setApp({ ...app, name: e.target.value });
  const updateCategory = (e: React.ChangeEvent<HTMLInputElement>) =>
    setApp({ ...app, category: e.target.value || null });
  const updateIdentifier = (e: React.ChangeEvent<HTMLInputElement>) => setApp({ ...app, identifier: e.target.value });

  const onSelectKind = (value: ApplicationIdentifier) => setApp({ ...app, kind: value });
  const onSelectMatchingStrategy = (value: MatchingStrategy) => setApp({ ...app, matchingStrategy: value });

  const onSelectMonitor = (value: number | null) => setApp({ ...app, monitor: value });
  const onSelectWorkspace = (value: string | null) => setApp({ ...app, workspace: value });

  const onChangeOption = (option: ApplicationOptions, value: boolean) => setApp({ ...app, [option]: value });

  const resetInvisibleBorders = () => setApp({ ...app, invisibleBorders: null });
  const onChangeInvisibleBorders = (side: keyof Rect.plain, value: number | null) => {
    setApp({
      ...app,
      invisibleBorders: {
        ...(app.invisibleBorders || new Rect().plain()),
        [side]: value || 0,
      },
    });
  };

  const monitorsOptions = monitors.map((_, i) => ({ label: `Monitor ${i + 1}`, value: i }));
  const workspaceOptions =
    app.monitor != null && monitors[app.monitor]
      ? monitors[app.monitor]?.workspaces.map(({ name }) => ({ label: name, value: name }))
      : [];

  return (
    <Modal
      title={`${isNew ? 'Creating' : 'Editing'} ${app.name}`}
      open={open}
      onCancel={onCancel}
      onOk={onInternalSave}
      okText={`${isNew ? 'Create' : 'Update'}`}
      centered
      className={cs.editModal}
    >
      <SettingsGroup>
        <div>
          <SettingsOption>
            <span>Name</span>
            <Input value={app.name} onChange={updateName} required />
          </SettingsOption>
          <SettingsOption>
            <span>Category</span>
            <Input value={app.category || ''} placeholder="None" onChange={updateCategory} />
          </SettingsOption>
        </div>
        <SettingsSubGroup label="Application Identifier">
          <SettingsOption>
            <span>Identifier</span>
            <Input value={app.identifier} onChange={updateIdentifier} />
          </SettingsOption>
          <SettingsOption>
            <span>Identify By</span>
            <Select value={app.kind} options={OptionsFromEnum(ApplicationIdentifier)} onSelect={onSelectKind} />
          </SettingsOption>
          <SettingsOption>
            <span>Maching Strategy</span>
            <Select
              value={app.matchingStrategy}
              options={OptionsFromEnum(MatchingStrategy)}
              onSelect={onSelectMatchingStrategy}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Binding *note: both options are required">
          <SettingsOption>
            <span>Monitor</span>
            <Select
              value={app.monitor}
              placeholder="None"
              allowClear
              options={monitorsOptions}
              onChange={onSelectMonitor}
            />
          </SettingsOption>
          <SettingsOption>
            <span>Workspace</span>
            <Select
              value={app.workspace}
              placeholder="None"
              allowClear
              options={workspaceOptions}
              onChange={onSelectWorkspace}
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Extra Options">
          {Object.values(ApplicationOptions).map((value, i) => (
            <SettingsOption key={i}>
              <span>{value}</span>
              <Switch value={app[value]} onChange={onChangeOption.bind(this, value)} />
            </SettingsOption>
          ))}
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption>
              <span>Specifit invisible borders</span>
              <Button type="dashed" onClick={resetInvisibleBorders}>
                ⟳
              </Button>
            </SettingsOption>
          }
        >
          <SettingsOption>
            <span>Left</span>
            <InputNumber
              value={invisibleBorders?.left}
              onChange={onChangeInvisibleBorders.bind(this, 'left')}
              placeholder="Global"
            />
          </SettingsOption>
          <SettingsOption>
            <span>Top</span>
            <InputNumber value={invisibleBorders?.top} onChange={onChangeInvisibleBorders.bind(this, 'top')} placeholder="Global" />
          </SettingsOption>
          <SettingsOption>
            <span>Right</span>
            <InputNumber
              value={invisibleBorders?.right}
              onChange={onChangeInvisibleBorders.bind(this, 'right')}
              placeholder="Global"
            />
          </SettingsOption>
          <SettingsOption>
            <span>Bottom</span>
            <InputNumber
              value={invisibleBorders?.bottom}
              onChange={onChangeInvisibleBorders.bind(this, 'bottom')}
              placeholder="Global"
            />
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>
    </Modal>
  );
};
