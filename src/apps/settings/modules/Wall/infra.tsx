import { convertFileSrc } from '@tauri-apps/api/core';
import { Button, InputNumber, Switch } from 'antd';
import { Reorder } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { SeelenWallWallpaper } from 'seelen-core';

import { dialog } from '../shared/tauri/infra';

import { newSelectors, RootActions } from '../shared/store/app/reducer';
import { Icon } from 'src/apps/shared/components/Icon';

import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import cs from './index.module.css';

export function WallSettings() {
  const wall = useSelector(newSelectors.wall);
  const { enabled, backgrounds, interval } = wall;

  const dispatch = useDispatch();
  const { t } = useTranslation();

  function onChangeEnabled(value: boolean) {
    dispatch(RootActions.setWall({ ...wall, enabled: value }));
  }

  function onChangeInterval(interval: number | null) {
    if (interval) {
      dispatch(RootActions.setWall({ ...wall, interval }));
    }
  }

  function onChangeBackgrounds(backgrounds: SeelenWallWallpaper[]) {
    dispatch(RootActions.setWall({ ...wall, backgrounds }));
  }

  async function onAddBackgrounds() {
    let newBackgrounds: SeelenWallWallpaper[] = [];

    const files = await dialog.open({
      multiple: true,
      title: t('wall.select'),
      filters: [
        { name: 'Media', extensions: ['jpg', 'jpeg', 'png', 'webp', 'gif', 'mp4', 'mkv', 'wav'] },
      ],
    });

    if (!files) {
      return;
    }

    for (const file of [files].flat()) {
      newBackgrounds.push({ ...new SeelenWallWallpaper(), path: file });
    }

    onChangeBackgrounds([...backgrounds, ...newBackgrounds]);
  }

  function onRemoveBackground(idx: number) {
    let newBackgrounds = [...backgrounds];
    newBackgrounds.splice(idx, 1);
    onChangeBackgrounds(newBackgrounds);
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t('wall.enable')}</b>
          <Switch value={enabled} onChange={onChangeEnabled} />
        </SettingsOption>
        <SettingsOption>
          <b>{t('wall.interval')}</b>
          <InputNumber value={interval} onChange={onChangeInterval} min={1} />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <b>{t('wall.backgrounds')}</b>
        {!!backgrounds.length && (
          <Reorder.Group
            values={backgrounds}
            onReorder={onChangeBackgrounds}
            className={cs.backgroundList}
            axis="y"
          >
            {backgrounds.map((bg, idx) => {
              let is_video = ['mp4', 'mkv', 'wav'].some((ext) => bg.path.endsWith(ext));

              return (
                <Reorder.Item key={bg.id} value={bg} className={cs.background}>
                  {is_video ? (
                    <div className={cs.video}>
                      <Icon iconName="FaVideo" />
                    </div>
                  ) : (
                    <img className={cs.image} src={convertFileSrc(bg.path)} />
                  )}
                  <b>{bg.path.split('\\').pop()}</b>
                  <Button type="primary" onClick={() => onRemoveBackground(idx)}>
                    <Icon iconName="IoTrash" size={14} />
                  </Button>
                </Reorder.Item>
              );
            })}
          </Reorder.Group>
        )}
        <SettingsOption>
          <div>{!backgrounds.length && t('wall.no_background')}</div>
          <Button type="primary" className={cs.backgroundAdd} onClick={onAddBackgrounds}>
            <Icon iconName="MdLibraryAdd" size={14} />
          </Button>
        </SettingsOption>
      </SettingsGroup>
    </>
  );
}
