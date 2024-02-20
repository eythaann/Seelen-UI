import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';
import { Button, Input, InputNumber, Modal, Select, Switch } from 'antd';
import { useState } from 'react';
import { useDispatch } from 'react-redux';

import cs from './infra.module.css';

import { useAppSelector, useDispatchCallback } from '../../shared/app/hooks';
import { Rect } from '../../shared/app/Rect';
import { getMonitorSelector, getWorkspaceSelector } from '../../shared/app/selectors';
import { OptionsFromEnum } from '../../shared/app/utils';
import { MonitorsActions } from './app';

import { Layout } from '../layouts/domain';

interface Props {
  workspaceIdx: number;
  monitorIdx: number;
}

export const AdvancedConfig = ({ workspaceIdx, monitorIdx }: Props) => {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const workspace = useAppSelector(getWorkspaceSelector(workspaceIdx, monitorIdx));
  const { workAreaOffset } = useAppSelector(getMonitorSelector(monitorIdx))!;

  const dispatch = useDispatch();

  if (!workspace) {
    return;
  }

  const showModal = () => {
    setIsModalOpen(true);
  };

  const handleOk = () => {
    setIsModalOpen(false);
  };

  const handleCancel = () => {
    setIsModalOpen(false);
  };

  const toggleLayoutRules = useDispatchCallback((value: boolean) => {
    if (value) {
      dispatch(MonitorsActions.enableLayoutRules({ monitorIdx, workspaceIdx }));
      return;
    }
    dispatch(MonitorsActions.disableLayoutRules({ monitorIdx, workspaceIdx }));
  });

  const toggleCustomLayoutRules = useDispatchCallback((value: boolean) => {
    if (value) {
      dispatch(MonitorsActions.enableCustomLayoutRules({ monitorIdx, workspaceIdx }));
      return;
    }
    dispatch(MonitorsActions.disableCustomLayoutRules({ monitorIdx, workspaceIdx }));
  });

  const resetOffset = () => dispatch(MonitorsActions.updateMonitor({ monitorIdx, key: 'workAreaOffset', value: null }));
  const onChangeOffset = (side: keyof Rect.plain, value: number | null) => {
    dispatch(
      MonitorsActions.updateMonitor({
        monitorIdx,
        key: 'workAreaOffset',
        value: {
          ...(workAreaOffset || new Rect().plain()),
          [side]: value || 0,
        },
      }),
    );
  };

  const onChangeLayoutRule = (key: string, value: Layout | null) => {
    dispatch(
      MonitorsActions.updateWorkspace({
        monitorIdx,
        workspaceIdx,
        key: 'layoutRules',
        value: {
          ...workspace.layoutRules,
          [key]: value,
        },
      }),
    );
  };

  const onChangeCustomLayout = (event: React.ChangeEvent<HTMLInputElement>) => {
    dispatch(
      MonitorsActions.updateWorkspace({
        monitorIdx,
        workspaceIdx,
        key: 'customLayout',
        value: event.target.value,
      }),
    );
  };
  const onChangeCustomLayoutRule = (key: string, event: React.ChangeEvent<HTMLInputElement>) => {
    dispatch(
      MonitorsActions.updateWorkspace({
        monitorIdx,
        workspaceIdx,
        key: 'customLayoutRules',
        value: {
          ...workspace.customLayoutRules,
          [key]: event.target.value,
        },
      }),
    );
  };

  const layoutRules = workspace.layoutRules
    ? Object.keys(workspace.layoutRules).map((key) => (
      <SettingsOption>
        <span>{key}</span>
        <Select
          value={workspace.layoutRules![key]!}
          options={OptionsFromEnum(Layout)}
          allowClear
          onChange={onChangeLayoutRule.bind(this, key)}
        />
      </SettingsOption>
    ))
    : [];

  const customLayoutRules = workspace.customLayoutRules
    ? Object.keys(workspace.customLayoutRules).map((key) => (
      <SettingsOption>
        <span>{key}</span>
        <Input
          value={workspace.customLayoutRules?.[key] || ''}
          placeholder="custom layout path"
          onChange={onChangeCustomLayoutRule.bind(this, key)}
        />
      </SettingsOption>
    ))
    : [];

  return (
    <>
      <Button className={cs.advancedTrigger} onClick={showModal}>
        ⚙️ Advanced
      </Button>
      <Modal
        title={`Editing: Monitor ${monitorIdx + 1}, ${workspace.name}`}
        onCancel={handleCancel}
        onOk={handleOk}
        open={isModalOpen}
        centered
        footer={null}
      >
        <div className={cs.advancedModal}>
          <SettingsGroup>
            <SettingsSubGroup
              label={
                <SettingsOption>
                  <span>Specifit monitor offsets (margins)</span>
                  <Button type="dashed" onClick={resetOffset}>
                    ⟳
                  </Button>
                </SettingsOption>
              }
            >
              <SettingsOption>
                <span>Left</span>
                <InputNumber
                  value={workAreaOffset?.left}
                  onChange={onChangeOffset.bind(this, 'left')}
                  placeholder="Global"
                />
              </SettingsOption>
              <SettingsOption>
                <span>Top</span>
                <InputNumber
                  value={workAreaOffset?.top}
                  onChange={onChangeOffset.bind(this, 'top')}
                  placeholder="Global"
                />
              </SettingsOption>
              <SettingsOption>
                <span>Right</span>
                <InputNumber
                  value={workAreaOffset?.right}
                  onChange={onChangeOffset.bind(this, 'right')}
                  placeholder="Global"
                />
              </SettingsOption>
              <SettingsOption>
                <span>Bottom</span>
                <InputNumber
                  value={workAreaOffset?.bottom}
                  onChange={onChangeOffset.bind(this, 'bottom')}
                  placeholder="Global"
                />
              </SettingsOption>
            </SettingsSubGroup>
          </SettingsGroup>

          <SettingsGroup>
            <SettingsOption>
              <span>{workspace.name} Custom Layout</span>
              <Input value={workspace.customLayout || ''} placeholder="custom layout path" onChange={onChangeCustomLayout} />
            </SettingsOption>
          </SettingsGroup>

          <SettingsGroup>
            <SettingsSubGroup
              label={
                <SettingsOption>
                  <span>{workspace.name} Layout Rules</span>
                  <Switch onChange={toggleLayoutRules} value={!!layoutRules.length} />
                </SettingsOption>
              }
            >
              {layoutRules}
            </SettingsSubGroup>
            <SettingsSubGroup
              label={
                <SettingsOption>
                  <span>{workspace.name} Custom Layout Rules</span>
                  <Switch onChange={toggleCustomLayoutRules} value={!!customLayoutRules.length} />
                </SettingsOption>
              }
            >
              {customLayoutRules}
            </SettingsSubGroup>
          </SettingsGroup>
        </div>
      </Modal>
    </>
  );
};
