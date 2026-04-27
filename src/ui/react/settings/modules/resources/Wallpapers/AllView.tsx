import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { ResourceKind, type Wallpaper } from "@seelen-ui/lib/types";
import { Icon } from "libs/ui/react/components/Icon/index.tsx";
import { Button, Input, Modal, Tooltip } from "antd";
import { useTranslation } from "react-i18next";
import { useEffect, useState } from "preact/hooks";
import { NavLink } from "react-router";

import cs from "../infra.module.css";

import { wallpapers } from "../../../state/resources.ts";
import {
  addWallpaperCollection,
  deleteWallpaperCollection,
  getWallpaperCollections,
  updateWallpaperCollection,
} from "../../Wall/application.ts";
import { WallpaperList } from "../../Wall/WallpaperList.tsx";

import { resolveDisplayName, ResourceCard, ResourceListHeader } from "../ResourceCard.tsx";
import { SettingsGroup, SettingsOption, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";

export function AllWallpapersView() {
  const { t, i18n } = useTranslation();
  const [search, setSearch] = useState("");
  const wallpaperCollections = getWallpaperCollections();

  const [editingCollectionId, setEditingCollectionId] = useState<string | null>(null);
  const [editingCollectionName, setEditingCollectionName] = useState("");
  const [isCreatingCollection, setIsCreatingCollection] = useState(false);
  const [newCollectionName, setNewCollectionName] = useState("");

  const editingCollection = editingCollectionId ? wallpaperCollections.find((c) => c.id === editingCollectionId) : null;

  useEffect(() => {
    if (editingCollection) {
      setEditingCollectionName(editingCollection.name);
    }
  }, [editingCollection]);

  const filtered = search
    ? wallpapers.value.filter((w) =>
      resolveDisplayName(w.metadata.displayName, i18n.language)
        .toLowerCase()
        .includes(search.toLowerCase())
    )
    : wallpapers.value;

  const handleCreateCollection = () => {
    setIsCreatingCollection(true);
    setNewCollectionName("");
  };

  const handleConfirmCreateCollection = () => {
    if (!newCollectionName.trim()) return;
    addWallpaperCollection({
      id: crypto.randomUUID(),
      name: newCollectionName.trim(),
      wallpapers: [],
    });
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
    updateWallpaperCollection({ ...editingCollection, name: editingCollectionName.trim() });
  };

  const handleCloseModal = () => {
    handleSaveCollectionName();
    setEditingCollectionId(null);
    setEditingCollectionName("");
  };

  const handleDeleteCollection = (id: string) => {
    deleteWallpaperCollection(id);
  };

  return (
    <>
      <ResourceListHeader
        discoverUrl="https://seelen.io/resources/s?category=Wallpaper"
        search={search}
        onSearch={setSearch}
      >
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <b>{t("resources.import_wallpapers")}</b>
          <Button
            type="default"
            onClick={() => {
              invoke(SeelenCommand.StateRequestWallpaperAddition);
            }}
          >
            <Icon iconName="MdLibraryAdd" />
          </Button>
        </div>
      </ResourceListHeader>

      <SettingsGroup>
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
            ? <div style={{ padding: "16px", textAlign: "center" }}>{t("wall.no_collections")}</div>
            : (
              wallpaperCollections.map((collection) => (
                <SettingsOption
                  key={collection.id}
                  label={
                    <span>
                      {collection.name} ({collection.wallpapers.length})
                    </span>
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
              ))
            )}
        </SettingsSubGroup>
      </SettingsGroup>

      <div className={cs.list}>
        {filtered.map((resource) => <WallpaperItem key={resource.id} resource={resource} />)}
      </div>

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

function WallpaperItem({ resource }: { resource: Wallpaper }) {
  return (
    <ResourceCard
      resource={resource}
      kind={ResourceKind.Wallpaper}
      actions={
        <NavLink to={`/wallpaper?${new URLSearchParams({ id: resource.id })}`}>
          <Button type="text">
            <Icon iconName="RiSettings4Fill" />
          </Button>
        </NavLink>
      }
    />
  );
}
