import { type AppConfig, AppExtraFlag } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Input, Modal, Switch, Table, Tooltip } from "antd";
import type { ColumnsType, ColumnType } from "antd/es/table";
import type { TFunction } from "i18next";
import { cloneDeep, debounce } from "lodash";
import { type ChangeEvent, useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { cx } from "../../shared/utils/app.ts";
import { getSorterByBool, getSorterByText } from "../app/filters.ts";

import type { AppConfigurationExtended } from "../domain.ts";

import { EditAppModal } from "./EditModal.tsx";
import cs from "./index.module.css";
import { appsConfigs } from "../../../state/resources.ts";
import { actions } from "../app/reducer.ts";

const ReadonlySwitch = (value: boolean, record: AppConfigurationExtended, _index: number) => {
  return (
    <Switch
      value={value}
      disabled
      className={cx({
        [cs.readonly!]: record.isBundled,
      })}
    />
  );
};

const getColumns = (t: TFunction): ColumnsType<AppConfigurationExtended> => {
  return [
    {
      title: t("apps_configurations.app.name"),
      dataIndex: "name",
      key: "name",
      fixed: "left",
      width: 120,
      sorter: getSorterByText("name"),
      render: (name) => (
        <Tooltip placement="topLeft" title={name}>
          {name}
        </Tooltip>
      ),
    },
    {
      title: t("apps_configurations.app.category"),
      dataIndex: "category",
      key: "category",
      width: 120,
      render(value, _record, _index) {
        return value || "-";
      },
      sorter: getSorterByText("category"),
    },
    ...Object.values(AppExtraFlag)
      .filter((option) => option !== AppExtraFlag.Unknown)
      .map(
        (option) => ({
          title: t(`apps_configurations.app.options.${option}`),
          dataIndex: option,
          key: option,
          align: "center",
          width: 140,
          render: ReadonlySwitch,
          sorter: getSorterByBool(option),
        } as ColumnType<AppConfigurationExtended>),
      ),
    {
      title: <ActionsTitle />,
      key: "operation",
      fixed: "right",
      align: "center",
      width: 60,
      render: (_, record, index) => <Actions record={record} index={index} />,
    },
  ];
};

function ActionsTitle() {
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { t } = useTranslation();

  const showModal = () => setIsModalOpen(true);
  const onCancel = () => setIsModalOpen(false);
  const onSave = (app: AppConfig) => {
    actions.push([app]);
    setIsModalOpen(false);
  };

  return (
    <div>
      <EditAppModal open={isModalOpen} isNew onSave={onSave} onCancel={onCancel} />
      <Button className={cs.newBtn} type="primary" onClick={showModal}>
        {t("apps_configurations.new")}
      </Button>
    </div>
  );
}

function Actions({ record }: { record: AppConfigurationExtended; index: number }) {
  const [isModalOpen, setIsModalOpen] = useState(false);

  const showModal = () => setIsModalOpen(true);
  const onCancel = () => setIsModalOpen(false);
  const onSave = (app: AppConfigurationExtended) => {
    if (record.isBundled) {
      let newApp = cloneDeep(app);
      newApp.isBundled = false;
      actions.push([newApp]);
    } else {
      actions.replace({ idx: record.key, app });
    }
    setIsModalOpen(false);
  };

  return (
    <div className={cs.actions}>
      {isModalOpen && (
        <EditAppModal
          open
          idx={record.isBundled ? undefined : record.key}
          onSave={onSave}
          onCancel={onCancel}
          readonlyApp={record.isBundled ? record : undefined}
        />
      )}
      <Button type={record.isBundled ? "default" : "primary"} onClick={showModal}>
        {record.isBundled ? <Icon iconName="FaEye" /> : <Icon iconName="MdOutlineEdit" />}
      </Button>
    </div>
  );
}

export function AppsConfiguration() {
  const [delay, setDelay] = useState(300);
  const [loading, setLoading] = useState(true);
  const [selectedAppsKey, setSelectedAppsKey] = useState<number[]>([]);
  const [searched, setSearched] = useState("");
  const [data, setData] = useState<AppConfigurationExtended[]>([]);

  const apps = appsConfigs.value;

  useEffect(() => {
    const data: AppConfigurationExtended[] = [];

    apps.forEach((app, index) => data.unshift({ ...app, key: index }));

    setTimeout(() => {
      setData(
        data.filter((app) => {
          return (
            app.name.toLowerCase().includes(searched) ||
            app.identifier.id.toLowerCase().includes(searched) ||
            app.identifier.and.some((id) => id.id.toLowerCase().includes(searched)) ||
            app.identifier.or.some((id) => id.id.toLowerCase().includes(searched))
          );
        }),
      );
      setLoading(false);
      setDelay(0);
    }, delay);
  }, [apps, searched]);

  const { t } = useTranslation();

  const performSwap = useCallback(() => {
    actions.swap(selectedAppsKey as [number, number]);
  }, [selectedAppsKey]);

  const confirmDelete = useCallback(() => {
    const modal = Modal.confirm({
      title: t("apps_configurations.confirm_delete_title"),
      content: t("apps_configurations.confirm_delete"),
      okText: t("delete"),
      onOk: () => {
        actions.deleteMany(selectedAppsKey);
        setSelectedAppsKey([]);
        modal.destroy();
      },
      okButtonProps: { danger: true },
      cancelText: t("cancel"),
      centered: true,
    });
  }, [selectedAppsKey]);

  const onSearch = useCallback(
    debounce((e: ChangeEvent<HTMLInputElement>) => {
      setSearched(e.currentTarget.value.toLowerCase());
    }, 200),
    [],
  );

  const columns = getColumns(t);
  columns[0]!.title = (
    <Input
      onChange={onSearch}
      onClick={(e) => e.stopPropagation()}
      placeholder={t("apps_configurations.search")}
    />
  );

  return (
    <div className={cs.container}>
      <Table
        loading={loading}
        dataSource={data}
        columns={columns}
        pagination={{ pageSize: 25 }}
        scroll={{ y: "calc(100vh - 150px)", x: "100vw" }}
        className={cs.table}
        rowSelection={{
          selectedRowKeys: selectedAppsKey,
          onChange(selectedRowKeys, _selectedRows, _info) {
            setSelectedAppsKey(selectedRowKeys as number[]);
          },
          getCheckboxProps(record) {
            return {
              disabled: record.isBundled,
            };
          },
        }}
      />
      <div className={cs.footer}>
        {
          /* <Button onClick={importApps}>{t("apps_configurations.import")}</Button>
        <Button onClick={exportApps} disabled={!selectedAppsKey.length}>
          {t("apps_configurations.export")}
        </Button> */
        }
        <Button type="primary" danger disabled={!selectedAppsKey.length} onClick={confirmDelete}>
          {t("apps_configurations.delete")}
        </Button>
        <Button onClick={performSwap} type="primary" disabled={selectedAppsKey.length !== 2}>
          {t("apps_configurations.swap")}
        </Button>
      </div>
    </div>
  );
}
