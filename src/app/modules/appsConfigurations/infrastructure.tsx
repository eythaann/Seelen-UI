import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { Button, Input, Modal, Select, Switch, Table } from 'antd';
import { ColumnsType } from 'antd/es/table';
import { useState } from 'react';

import { OptionsFromEnum } from '../shared/app/utils';

import {
  AppConfiguration,
  ApplicationIdentifier,
  ApplicationOptions,
  MatchingStrategy,
} from './domain';

import cs from './index.module.css';

const data: AppConfiguration[] = [];
for (let i = 0; i < 240; i++) {
  data.push({ ...AppConfiguration.default(), key: i });
}

const ReadonlySwitch = (value: boolean, _record: AppConfiguration, _index: number) => {
  return <Switch value={value} disabled />;
};

const columns: ColumnsType<AppConfiguration> = [
  {
    title: 'Name',
    dataIndex: 'name',
    key: 'name',
    fixed: 'left',
    width: 100,
    sorter: (a, b) => {
      const nameA = a.name.toLowerCase();
      const nameB = b.name.toLowerCase();
      if (nameA < nameB) {
        return -1;
      }
      if (nameA > nameB) {
        return 1;
      }
      return 0;
    },
  },
  {
    title: 'Category',
    dataIndex: 'category',
    key: 'category',
    width: 100,
    render(value, _record, _index) {
      return value || 'None';
    },
  },
  {
    title: 'Workspace',
    dataIndex: 'workspace',
    key: 'workspace',
    width: 100,
    render(value, _record, _index) {
      return value || 'None';
    },
  },
  {
    title: 'Identify By',
    dataIndex: 'kind',
    key: 'kind',
    width: 100,
  },
  {
    title: 'Identifier',
    dataIndex: 'identifier',
    key: 'identifier',
    width: 100,
  },
  {
    title: 'Forced',
    dataIndex: ApplicationOptions.Force,
    key: ApplicationOptions.Force,
    align: 'center',
    width: 80,
    render: ReadonlySwitch,
  },
  {
    title: 'Float',
    dataIndex: ApplicationOptions.Float,
    key: ApplicationOptions.Float,
    align: 'center',
    width: 80,
    render: ReadonlySwitch,
  },
  {
    title: 'Unmanaged',
    dataIndex: ApplicationOptions.Unmanage,
    key: ApplicationOptions.Unmanage,
    align: 'center',
    width: 100,
    render: ReadonlySwitch,
  },
  {
    title: 'Border Overflow',
    dataIndex: ApplicationOptions.BorderOverflow,
    key: ApplicationOptions.BorderOverflow,
    align: 'center',
    width: 150,
    render: ReadonlySwitch,
  },
  {
    title: 'Layered',
    dataIndex: ApplicationOptions.Layered,
    key: ApplicationOptions.Layered,
    align: 'center',
    width: 90,
    render: ReadonlySwitch,
  },
  {
    title: 'Object Name Change',
    dataIndex: ApplicationOptions.ObjectNameChange,
    key: ApplicationOptions.ObjectNameChange,
    align: 'center',
    width: 180,
    render: ReadonlySwitch,
  },
  {
    title: 'Tray and MultiWindow',
    dataIndex: ApplicationOptions.TrayAndMultiWindow,
    key: ApplicationOptions.TrayAndMultiWindow,
    align: 'center',
    width: 180,
    render: ReadonlySwitch,
  },
  {
    title: (
      <div>
        <Button className={cs.newBtn} type="primary">
          New
        </Button>
      </div>
    ),
    key: 'operation',
    fixed: 'right',
    align: 'center',
    width: 85,
    render: (_, record, index) => <Actions record={record} index={index} />,
  },
];

function Actions({ record, index }: { record: AppConfiguration; index: number }) {
  const [isModalOpen, setIsModalOpen] = useState(false);

  const showModal = () => {
    setIsModalOpen(true);
  };

  const handleOk = () => {
    setIsModalOpen(false);
  };

  const handleCancel = () => {
    setIsModalOpen(false);
  };

  const confirm = () => {
    Modal.confirm({
      title: 'Confirm Delete',
      content: 'Sure on delete this application?',
      okText: 'delete',
      okButtonProps: { danger: true },
      cancelText: 'cancel',
      centered: true,
    });
  };

  return (
    <div className={cs.actions}>
      <Modal
        title={`Editing: ${record.name}`}
        open={isModalOpen}
        onCancel={handleCancel}
        onOk={handleOk}
        centered
      >
        <SettingsGroup>
          <div>
            <SettingsOption>
              <span>Name</span>
              <Input value={record.name} />
            </SettingsOption>
            <SettingsOption>
              <span>Category</span>
              <Input value={record.category || ''} placeholder="None" />
            </SettingsOption>
            <SettingsOption>
              <span>Workspace</span>
              <Input value={record.workspace || ''} placeholder="None" />
            </SettingsOption>
          </div>
          <SettingsSubGroup label="Application Identifier">
            <SettingsOption>
              <span>Identifier</span>
              <Input value={record.identifier} />
            </SettingsOption>
            <SettingsOption>
              <span>Identify By</span>
              <Select value={record.kind} options={OptionsFromEnum(ApplicationIdentifier)} />
            </SettingsOption>
            <SettingsOption>
              <span>Maching Strategy</span>
              <Select value={record.machingStrategy} options={OptionsFromEnum(MatchingStrategy)} />
            </SettingsOption>
          </SettingsSubGroup>
        </SettingsGroup>
        <SettingsGroup>
          <SettingsSubGroup label="Extra Options">
            {Object.values(ApplicationOptions).map((value) => (
              <SettingsOption>
                <span>{value}</span>
                <Switch value={record[value]} />
              </SettingsOption>
            ))}
          </SettingsSubGroup>
        </SettingsGroup>
      </Modal>
      <Button type="primary" onClick={showModal}>
        ✏️
      </Button>
      <Button danger onClick={confirm}>
        ❌
      </Button>
    </div>
  );
}

export function AppsConfiguration() {
  return (
    <Table
      dataSource={data}
      columns={columns}
      pagination={{ defaultPageSize: 20 }}
      scroll={{ y: 350, x: '100vw' }}
      className={cs.table}
    />
  );
}
