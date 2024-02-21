import { EditAppModal } from './EditModal';
import { createSelector } from '@reduxjs/toolkit';
import { Button, Modal, Switch, Table } from 'antd';
import { ColumnsType, ColumnType } from 'antd/es/table';
import { useState } from 'react';
import { useDispatch } from 'react-redux';

import { useAppSelector } from '../../shared/app/hooks';
import { RootSelectors } from '../../shared/app/selectors';
import { getSorterByBool, getSorterByText } from '../app/filters';
import { AppsConfigActions } from '../app/reducer';

import {
  AppConfiguration,
  ApplicationOptions,
  LabelByAppOption,
} from '../domain';

import cs from './index.module.css';

const ReadonlySwitch = (value: boolean, _record: AppConfigWithKey, _index: number) => {
  return <Switch value={value} disabled />;
};

type AppConfigWithKey = AppConfiguration & { key: number };
const columns: ColumnsType<AppConfigWithKey> = [
  {
    title: 'Name',
    dataIndex: 'name',
    key: 'name',
    fixed: 'left',
    width: 120,
    sorter: getSorterByText('name'),
  },
  {
    title: 'Category',
    dataIndex: 'category',
    key: 'category',
    width: 120,
    render(value, _record, _index) {
      return value || '-';
    },
    sorter: getSorterByText('category'),
  },
  {
    title: 'Monitor',
    dataIndex: 'monitor',
    key: 'monitor',
    width: 120,
    render(value, _record, _index) {
      return value != null ? `Monitor ${value + 1}` : '-';
    },
    sorter: getSorterByText('monitor'),
  },
  {
    title: 'Workspace',
    dataIndex: 'workspace',
    key: 'workspace',
    width: 120,
    render(value, _record, _index) {
      return value || '-';
    },
    sorter: getSorterByText('workspace'),
  },
  {
    title: 'Identifier',
    dataIndex: 'identifier',
    key: 'identifier',
    width: 120,
    sorter: getSorterByText('identifier'),
  },
  {
    title: 'By',
    dataIndex: 'kind',
    key: 'kind',
    width: 80,
    align: 'center',
    sorter: getSorterByText('kind'),
  },
  {
    title: 'Strategy',
    dataIndex: 'matchingStrategy',
    key: 'matchingStrategy',
    width: 110,
    align: 'center',
    sorter: getSorterByText('matchingStrategy'),
  },
  ...Object.values(ApplicationOptions).map((option) => ({
    title: LabelByAppOption[option],
    dataIndex: option,
    key: option,
    align: 'center',
    width: 140,
    render: ReadonlySwitch,
    sorter: getSorterByBool(option),
  } as ColumnType<AppConfigWithKey>)),
  {
    title: <ActionsTitle />,
    key: 'operation',
    fixed: 'right',
    align: 'center',
    width: 80,
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

function Actions({ record }: { record: AppConfigWithKey; index: number }) {
  const [isModalOpen, setIsModalOpen] = useState(false);
  const dispatch = useDispatch();

  const showModal = () => setIsModalOpen(true);
  const onCancel = () => setIsModalOpen(false);
  const onSave = (app: AppConfiguration) => {
    dispatch(AppsConfigActions.replace({ idx: record.key, app }));
    setIsModalOpen(false);
  };

  const confirm = () => {
    const modal = Modal.confirm({
      title: 'Confirm Delete',
      content: 'Sure on delete this application?',
      okText: 'delete',
      onOk: () => {
        dispatch(AppsConfigActions.delete(record.key));
        modal.destroy();
      },
      okButtonProps: { danger: true },
      cancelText: 'cancel',
      centered: true,
    });
  };

  return (
    <div className={cs.actions}>
      {isModalOpen && <EditAppModal open idx={record.key} onSave={onSave} onCancel={onCancel}/>}
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
  const apps = useAppSelector(createSelector(RootSelectors.appsConfigurations, (apps) => {
    return apps.map((app, index) => ({ ...app, key: index }));
  }));

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
