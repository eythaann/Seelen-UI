import { Button, InputNumber, Modal } from 'antd';
import { useState } from 'react';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../shared/utils/infra';
import cs from './infra.module.css';

import { getMonitorSelector, getWorkspaceSelector } from '../../shared/store/app/selectors';
import { Rect } from '../../shared/utils/app/Rect';
import { MonitorsActions } from './app';

import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../../components/SettingsBox';

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

  const resetOffset = () =>
    dispatch(MonitorsActions.updateMonitor({ monitorIdx, key: 'workAreaOffset', value: null }));
  const onChangeOffset = (side: keyof Rect, value: number | null) => {
    dispatch(
      MonitorsActions.updateMonitor({
        monitorIdx,
        key: 'workAreaOffset',
        value: {
          ...(workAreaOffset || new Rect().toJSON()),
          [side]: value || 0,
        },
      }),
    );
  };

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
        </div>
      </Modal>
    </>
  );
};
