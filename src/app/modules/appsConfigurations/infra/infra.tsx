import { EditAppModal } from './EditModal';
import { Button, Modal, Switch, Table } from 'antd';
import { ColumnsType } from 'antd/es/table';
import { useState } from 'react';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../shared/app/hooks';
import { RootSelectors } from '../../shared/app/selectors';
import { AppsConfigActions } from '../app';

import {
  AppConfiguration,
  ApplicationOptions,
} from '../domain';

import cs from './index.module.css';

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
      return value || '-';
    },
  },
  {
    title: 'Monitor',
    dataIndex: 'monitor',
    key: 'monitor',
    width: 70,
    render(value, _record, _index) {
      return value ?? '-';
    },
  },
  {
    title: 'Workspace',
    dataIndex: 'workspace',
    key: 'workspace',
    width: 100,
    render(value, _record, _index) {
      return value || '-';
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
    title: <ActionsTitle />,
    key: 'operation',
    fixed: 'right',
    align: 'center',
    width: 85,
    render: (_, record, index) => <Actions record={record} index={index} />,
  },
];

function ActionsTitle() {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const dispatch = useDispatch();

  const showModal = () => setIsModalOpen(true);
  const onCancel = () => setIsModalOpen(false);
  const onSave = (app: AppConfiguration) => {
    dispatch(AppsConfigActions.push(app));
    setIsModalOpen(false);
  };

  return (
    <div>
      <EditAppModal open={isModalOpen} isNew onSave={onSave} onCancel={onCancel}/>
      <Button className={cs.newBtn} type="primary" onClick={showModal}>
        New
      </Button>
    </div>
  );
}

function Actions({ index }: { record: AppConfiguration; index: number }) {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const dispatch = useDispatch();

  const showModal = () => setIsModalOpen(true);
  const onCancel = () => setIsModalOpen(false);
  const onSave = (app: AppConfiguration) => {
    dispatch(AppsConfigActions.replace({ idx: index, app }));
    setIsModalOpen(false);
  };

  const confirm = () => {
    const modal = Modal.confirm({
      title: 'Confirm Delete',
      content: 'Sure on delete this application?',
      okText: 'delete',
      onOk: () => {
        dispatch(AppsConfigActions.delete(index));
        modal.destroy();
      },
      okButtonProps: { danger: true },
      cancelText: 'cancel',
      centered: true,
    });
  };

  return (
    <div className={cs.actions}>
      {isModalOpen && <EditAppModal open idx={index} onSave={onSave} onCancel={onCancel}/>}
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
  const apps = useAppSelector(RootSelectors.appsConfigurations);

  return (
    <Table
      dataSource={apps}
      columns={columns}
      pagination={{ defaultPageSize: 20 }}
      scroll={{ y: 350, x: '100vw' }}
      className={cs.table}
    />
  );
}
