import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Badge, Button, Input, InputNumber, Modal, Select, Switch, Tooltip } from "antd";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useDispatch, useSelector } from "react-redux";
import { Link } from "react-router";

import { newSelectors, RootActions } from "../shared/store/app/reducer.ts";

import { MultimonitorBehaviour } from "@seelen-ui/lib/types";
import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../components/SettingsBox/index.tsx";
import { WallpaperList } from "./WallpaperList.tsx";
import cs from "./index.module.css";

export function WallSettings() {
  const wall = useSelector(newSelectors.wall);
  const wallpaperCollections = useSelector(newSelectors.wallpaperCollections);
  const { enabled, interval } = wall;

  const [time, setTime] = useState({
    hours: Math.floor(interval / 3600),
    minutes: Math.floor((interval / 60) % 60),
  });

  const [editingCollectionId, setEditingCollectionId] = useState<string | null>(null);
  const [editingCollectionName, setEditingCollectionName] = useState("");
  const [isCreatingCollection, setIsCreatingCollection] = useState(false);
  const [newCollectionName, setNewCollectionName] = useState("");

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const editingCollection = editingCollectionId ? wallpaperCollections.find((c) => c.id === editingCollectionId) : null;

  useEffect(() => {
    setTime({
      hours: Math.floor(interval / 3600),
      minutes: Math.floor((interval / 60) % 60),
    });
  }, [interval]);

  useEffect(() => {
    if (editingCollection) {
      setEditingCollectionName(editingCollection.name);
    }
  }, [editingCollection]);

  const patchWallSettings = (changes: Partial<typeof wall>) => {
    dispatch(RootActions.patchWall({ ...changes }));
  };

  function onChangeEnabled(enabled: boolean) {
    patchWallSettings({ enabled });
  }

  const updateTime = (key: "hours" | "minutes", value: number | null) => {
    if (value === null) return;
    const newTime = { ...time, [key]: Math.floor(value) };
    setTime(newTime);
    const newInterval = Math.max(newTime.hours * 3600 + newTime.minutes * 60, 60);
    patchWallSettings({ interval: newInterval });
  };

  const handleCreateCollection = () => {
    setIsCreatingCollection(true);
    setNewCollectionName("");
  };

  const handleConfirmCreateCollection = () => {
    if (!newCollectionName.trim()) {
      return;
    }

    const newCollection = {
      id: crypto.randomUUID(),
      name: newCollectionName.trim(),
      wallpapers: [],
    };
    dispatch(RootActions.addWallpaperCollection(newCollection));
    setIsCreatingCollection(false);
    setNewCollectionName("");
  };

  const handleCancelCreateCollection = () => {
    setIsCreatingCollection(false);
    setNewCollectionName("");
  };

  const handleEditCollection = (id: string) => {
    setEditingCollectionId(id);
  };

  const handleSaveCollectionName = () => {
    if (!editingCollection || !editingCollectionName.trim()) return;

    dispatch(
      RootActions.updateWallpaperCollection({
        ...editingCollection,
        name: editingCollectionName.trim(),
      }),
    );
  };

  const handleCloseModal = () => {
    handleSaveCollectionName();
    setEditingCollectionId(null);
    setEditingCollectionName("");
  };

  const handleDeleteCollection = (id: string) => {
    dispatch(RootActions.deleteWallpaperCollection(id));
  };

  return (
    <>
      <SettingsGroup>
        <SettingsOption
          label={<b>{t("wall.enable")}</b>}
          action={<Switch value={enabled} onChange={onChangeEnabled} />}
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={<b>{t("wall.multimonitor_behaviour")}</b>}
          action={
            <Select
              style={{ width: 200 }}
              value={wall.multimonitorBehaviour}
              onChange={(value) => patchWallSettings({ multimonitorBehaviour: value })}
              options={[
                {
                  label: t("wall.per_monitor"),
                  value: MultimonitorBehaviour.PerMonitor,
                },
                {
                  label: t("wall.extend"),
                  value: MultimonitorBehaviour.Extend,
                },
              ]}
            />
          }
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={<b>{t("wall.random")}</b>}
          action={
            <Switch
              value={wall.randomize}
              onChange={(randomize) => patchWallSettings({ randomize })}
            />
          }
        />
        <SettingsOption
          label={<b>{t("wall.interval")}</b>}
          action={
            <div className={cs.interval}>
              {["hours", "minutes"].map((unit) => (
                <div key={unit}>
                  <b>{t(`wall.${unit}`)}:</b>
                  <InputNumber
                    value={time[unit as keyof typeof time]}
                    onChange={(value) => updateTime(unit as "hours" | "minutes", value)}
                    min={0}
                    style={{ width: 50 }}
                  />
                </div>
              ))}
            </div>
          }
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("wall.backgrounds")}
          action={
            <Link to="/resources/wallpaper">
              <Button type="primary">
                <Icon iconName="IoImages" />
              </Button>
            </Link>
          }
        />

        <SettingsSubGroup
          label={
            <SettingsOption
              label={t("wall.collections")}
              action={
                <Tooltip title={t("wall.create_collection")}>
                  <Button type="primary" onClick={handleCreateCollection}>
                    <Icon iconName="IoAdd" />
                  </Button>
                </Tooltip>
              }
            />
          }
        >
          {wallpaperCollections.length === 0
            ? (
              <div
                style={{ padding: "16px", textAlign: "center", color: "var(--config-text-muted)" }}
              >
                {t("wall.no_collections")}
              </div>
            )
            : (
              wallpaperCollections.map((collection) => {
                const isDefault = wall.defaultCollection === collection.id;
                return (
                  <SettingsOption
                    key={collection.id}
                    label={
                      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                        {isDefault && (
                          <Tooltip title={t("wall.default_collection")}>
                            <Badge status="success" />
                          </Tooltip>
                        )}
                        <span>
                          {collection.name}
                          <span
                            style={{
                              marginLeft: 4,
                              color: "var(--config-text-muted)",
                              fontSize: "0.9em",
                            }}
                          >
                            ({collection.wallpapers.length})
                          </span>
                        </span>
                      </div>
                    }
                    action={
                      <div style={{ display: "flex", gap: 8 }}>
                        <Tooltip title={t("wall.edit_collection")}>
                          <Button size="small" onClick={() => handleEditCollection(collection.id)}>
                            <Icon iconName="MdEdit" />
                          </Button>
                        </Tooltip>
                        <Tooltip title={t("wall.delete_collection")}>
                          <Button
                            size="small"
                            danger
                            onClick={() => handleDeleteCollection(collection.id)}
                          >
                            <Icon iconName="IoTrash" />
                          </Button>
                        </Tooltip>
                      </div>
                    }
                  />
                );
              })
            )}
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t("wall.default_collection")}
          action={
            <Select
              style={{ width: 200 }}
              value={wall.defaultCollection ?? undefined}
              onChange={(value) => dispatch(RootActions.setDefaultWallpaperCollection(value || null))}
              placeholder={t("wall.select_collection")}
              allowClear
            >
              {wallpaperCollections.map((collection) => (
                <Select.Option key={collection.id} value={collection.id}>
                  {collection.name}
                </Select.Option>
              ))}
            </Select>
          }
        />
      </SettingsGroup>

      <Modal
        title={t("wall.create_collection")}
        open={isCreatingCollection}
        onOk={handleConfirmCreateCollection}
        onCancel={handleCancelCreateCollection}
        okText={t("wall.create")}
        cancelText={t("wall.cancel")}
        okButtonProps={{ disabled: !newCollectionName.trim() }}
        centered
      >
        <Input
          placeholder={t("wall.collection_name")}
          value={newCollectionName}
          onChange={(e) => setNewCollectionName(e.currentTarget.value)}
          onPressEnter={handleConfirmCreateCollection}
          autoFocus
          style={{ marginTop: 16 }}
        />
      </Modal>

      <Modal
        title={
          <Input
            style={{ width: "80%" }}
            placeholder={t("wall.collection_name")}
            value={editingCollectionName}
            onChange={(e) => setEditingCollectionName(e.currentTarget.value)}
            onBlur={handleSaveCollectionName}
            onPressEnter={handleSaveCollectionName}
          />
        }
        open={!!editingCollectionId}
        onCancel={handleCloseModal}
        footer={
          <Button type="primary" onClick={handleCloseModal}>
            {t("wall.close")}
          </Button>
        }
        width={800}
        centered
      >
        {editingCollectionId && <WallpaperList collectionId={editingCollectionId} />}
      </Modal>
    </>
  );
}
