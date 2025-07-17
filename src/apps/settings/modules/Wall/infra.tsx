import { PlaybackSpeed, WallpaperId } from '@seelen-ui/lib/types';
import { Button, ColorPicker, InputNumber, Select, Slider, Switch } from 'antd';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { Link } from 'react-router';

import { newSelectors, RootActions } from '../shared/store/app/reducer';
import { Icon } from 'src/apps/shared/components/Icon';

import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { WallpaperList } from './WallpaperList';
import cs from './index.module.css';

const playbackSpeeds: PlaybackSpeed[] = [
  'xDot25',
  'xDot5',
  'xDot75',
  'x1',
  'x1Dot25',
  'x1Dot5',
  'x1Dot75',
  'x2',
];
const playbackSpeedOptions = playbackSpeeds.map((s) => ({
  label: s.replace('Dot', '.').replace('x.', 'x0.'),
  value: s,
}));

export function WallSettings() {
  const wall = useSelector(newSelectors.wall);
  const { enabled, interval } = wall;

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

  const patchWallSettings = (changes: Partial<typeof wall>) => {
    dispatch(RootActions.setWall({ ...wall, ...changes }));
  };

  function onChangeEnabled(enabled: boolean) {
    patchWallSettings({ enabled });
  }

  const updateTime = (key: 'hours' | 'minutes' | 'seconds', value: number | null) => {
    if (value === null) return;
    const newTime = { ...time, [key]: Math.floor(value) };
    setTime(newTime);
    const newInterval = Math.max(newTime.hours * 3600 + newTime.minutes * 60 + newTime.seconds, 1);
    patchWallSettings({ interval: newInterval });
  };

  function onChangeBackgrounds(ids: WallpaperId[]) {
    patchWallSettings({ backgroundsV2: ids });
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
          <Switch
            value={wall.randomize}
            onChange={(randomize) => patchWallSettings({ randomize })}
          />
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
        <SettingsOption
          label={t('wall.playback')}
          action={
            <Select
              value={wall.playbackSpeed}
              options={playbackSpeedOptions}
              onSelect={(playbackSpeed) => {
                patchWallSettings({ playbackSpeed });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.flipHorizontal')}
          action={
            <Switch
              value={wall.flipHorizontal}
              onChange={(flipHorizontal) => {
                patchWallSettings({ flipHorizontal });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.flipVertical')}
          action={
            <Switch
              value={wall.flipVertical}
              onChange={(flipVertical) => {
                patchWallSettings({ flipVertical });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.blur')}
          action={
            <Slider
              defaultValue={wall.blur}
              min={0}
              max={50}
              onChangeComplete={(blur) => {
                patchWallSettings({ blur });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.objectFit')}
          action={
            <Select
              value={wall.objectFit}
              options={[
                { label: t('wall.fit.cover'), value: 'cover' },
                { label: t('wall.fit.contain'), value: 'contain' },
                { label: t('wall.fit.fill'), value: 'fill' },
              ]}
              onSelect={(objectFit) => {
                patchWallSettings({ objectFit });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.objectPosition')}
          action={
            <Select
              value={wall.objectPosition}
              options={[
                { label: t('wall.position.top'), value: 'top' },
                { label: t('wall.position.center'), value: 'center' },
                { label: t('wall.position.bottom'), value: 'bottom' },
                { label: t('wall.position.left'), value: 'left' },
                { label: t('wall.position.right'), value: 'right' },
              ]}
              onSelect={(objectPosition) => {
                patchWallSettings({ objectPosition });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.saturation')}
          action={
            <Slider
              defaultValue={wall.saturation}
              min={0}
              step={0.01}
              max={2}
              onChangeComplete={(saturation) => {
                patchWallSettings({ saturation });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.contrast')}
          action={
            <Slider
              defaultValue={wall.contrast}
              min={0}
              step={0.01}
              max={2}
              onChangeComplete={(contrast) => {
                patchWallSettings({ contrast });
              }}
            />
          }
        />
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption
              label={t('wall.withOverlay')}
              action={
                <Switch
                  value={wall.withOverlay}
                  onChange={(withOverlay) => {
                    patchWallSettings({ withOverlay });
                  }}
                />
              }
            />
          }
        >
          <SettingsOption
            label={t('wall.overlayMixBlendMode')}
            action={
              <Select
                value={wall.overlayMixBlendMode}
                options={[
                  { label: 'normal', value: 'normal' },
                  { label: 'multiply', value: 'multiply' },
                  { label: 'screen', value: 'screen' },
                  { label: 'overlay', value: 'overlay' },
                  { label: 'darken', value: 'darken' },
                  { label: 'lighten', value: 'lighten' },
                  { label: 'color-dodge', value: 'color-dodge' },
                  { label: 'color-burn', value: 'color-burn' },
                  { label: 'hard-light', value: 'hard-light' },
                  { label: 'soft-light', value: 'soft-light' },
                  { label: 'difference', value: 'difference' },
                  { label: 'exclusion', value: 'exclusion' },
                  { label: 'hue', value: 'hue' },
                  { label: 'saturation', value: 'saturation' },
                  { label: 'color', value: 'color' },
                  { label: 'luminosity', value: 'luminosity' },
                  { label: 'plus-darker', value: 'plus-darker' },
                  { label: 'plus-lighter', value: 'plus-lighter' },
                ]}
                onSelect={(overlayMixBlendMode) => {
                  patchWallSettings({ overlayMixBlendMode });
                }}
              />
            }
          />

          <SettingsOption
            label={t('wall.overlayColor')}
            action={
              <ColorPicker
                showText
                value={wall.overlayColor}
                onChangeComplete={(color) => {
                  patchWallSettings({ overlayColor: color.toHexString() });
                }}
              />
            }
          />
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption>
          <b>{t('wall.backgrounds')}</b>
          <Link to="/resources/wallpaper">
            <Button type="primary" className={cs.backgroundAdd}>
              <Icon iconName="IoImages" size={14} />
            </Button>
          </Link>
        </SettingsOption>

        <WallpaperList enabled={wall.backgroundsV2} onChangeEnabled={onChangeBackgrounds} />
      </SettingsGroup>
    </>
  );
}
