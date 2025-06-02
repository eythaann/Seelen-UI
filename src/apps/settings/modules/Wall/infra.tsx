import { dialog } from '@seelen-ui/lib/tauri';
import { SeelenWallWallpaper } from '@seelen-ui/lib/types';
import { convertFileSrc } from '@tauri-apps/api/core';
import { Button, InputNumber, Switch } from 'antd';
import { Reorder } from 'framer-motion';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors, RootActions } from '../shared/store/app/reducer';
import { Icon } from 'src/apps/shared/components/Icon';

import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import cs from './index.module.css';

export function WallSettings() {
  const wall = useSelector(newSelectors.wall);
  const { enabled, interval, backgrounds } = wall;

  const [time, setTime] = useState({
    hours: Math.floor(interval / 3600),
    minutes: Math.floor((interval / 60) % 60),
    seconds: interval % 60,
  });

  const dispatch = useDispatch();
  const { t } = useTranslation();

  useEffect(() => {
    setTime({
      hours: Math.floor(interval / 3600),
      minutes: Math.floor((interval / 60) % 60),
      seconds: interval % 60,
    });
  }, [interval]);

  const updateWall = (changes: Partial<typeof wall>) => {
    dispatch(RootActions.setWall({ ...wall, ...changes }));
  };

  function onChangeEnabled(enabled: boolean) {
    updateWall({ enabled });
  }

  const updateTime = (key: 'hours' | 'minutes' | 'seconds', value: number | null) => {
    if (value === null) return;
    const newTime = { ...time, [key]: Math.floor(value) };
    setTime(newTime);
    const newInterval = Math.max(newTime.hours * 3600 + newTime.minutes * 60 + newTime.seconds, 1);
    updateWall({ interval: newInterval });
  };

  function onChangeBackgrounds(backgrounds: SeelenWallWallpaper[]) {
    updateWall({ backgrounds });
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
      newBackgrounds.push({ id: crypto.randomUUID(), path: file });
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
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t('wall.random')}</b>
          <Switch value={wall.randomize} onChange={(randomize) => updateWall({ randomize })} />
        </SettingsOption>
        <SettingsOption>
          <b>{t('wall.interval')}</b>
          <div className={cs.interval}>
            {['hours', 'minutes', 'seconds'].map((unit) => (
              <div key={unit}>
                <b>{t(`wall.${unit}`)}:</b>
                <InputNumber
                  value={time[unit as keyof typeof time]}
                  onChange={(value) => updateTime(unit as 'hours' | 'minutes' | 'seconds', value)}
                  min={0}
                  style={{ width: 50 }}
                />
              </div>
            ))}
          </div>
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t('wall.backgrounds')}</b>
          <Button type="primary" className={cs.backgroundAdd} onClick={onAddBackgrounds}>
            <Icon iconName="MdLibraryAdd" size={14} />
          </Button>
        </SettingsOption>
        {!!backgrounds.length ? (
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
        ) : (
          <div>{t('wall.no_background')}</div>
        )}
      </SettingsGroup>
    </>
  );
}
