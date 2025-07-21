import { WallpaperConfiguration } from '@seelen-ui/lib';
import { PlaybackSpeed, WallpaperId, WallpaperInstanceSettings } from '@seelen-ui/lib/types';
import { ResourceText } from '@shared/components/ResourceText';
import { Wallpaper } from '@shared/components/Wallpaper';
import { ColorPicker, Select, Slider, Switch } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';
import { useParams } from 'react-router';

import { newSelectors, RootActions } from '../../shared/store/app/reducer';
import {
  SettingsGroup,
  SettingsOption,
  SettingsSubGroup,
} from 'src/apps/settings/components/SettingsBox';

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

const defaultWallpaperConfig = await WallpaperConfiguration.default();

export function SingleWallpaperView() {
  const { username, resourceName } = useParams<'username' | 'resourceName'>();
  const resourceId = `@${username}/${resourceName}` as WallpaperId;

  const wallpaper = useSelector(newSelectors.wallpapers);
  const editingWallpaper = wallpaper.find((wallpaper) => wallpaper.id === resourceId);

  const storedSettings = useSelector(newSelectors.wall.byBackground);
  const config = { ...defaultWallpaperConfig, ...(storedSettings[resourceId] || {}) };

  const d = useDispatch();
  const { t } = useTranslation();

  if (!editingWallpaper) {
    return <div>Ups 404</div>;
  }

  function patchWallSettings(patch: Partial<WallpaperInstanceSettings>) {
    d(RootActions.patchWallpaperSettings({ id: resourceId, patch }));
  }

  return (
    <>
      <div
        style={{
          position: 'relative',
          width: '100%',
          aspectRatio: '16 / 9',
          backgroundColor: '#000',
          overflow: 'hidden',
          marginBottom: '10px',
          borderRadius: '10px',
        }}
      >
        <Wallpaper definition={editingWallpaper} config={config} />
      </div>

      <SettingsGroup>
        <b style={{ textAlign: 'center', fontSize: '1.1rem' }}>
          <ResourceText text={editingWallpaper.metadata.displayName} />
        </b>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsOption
          label={t('wall.playback')}
          action={
            <Select
              value={config.playbackSpeed}
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
              value={config.flipHorizontal}
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
              value={config.flipVertical}
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
              value={config.blur}
              min={0}
              max={50}
              onChange={(blur) => {
                patchWallSettings({ blur });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.objectFit')}
          action={
            <Select
              value={config.objectFit}
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
              value={config.objectPosition}
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
              value={config.saturation}
              min={0}
              step={0.01}
              max={2}
              onChange={(saturation) => {
                patchWallSettings({ saturation });
              }}
            />
          }
        />

        <SettingsOption
          label={t('wall.contrast')}
          action={
            <Slider
              value={config.contrast}
              min={0}
              step={0.01}
              max={2}
              onChange={(contrast) => {
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
                  value={config.withOverlay}
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
                value={config.overlayMixBlendMode}
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
                value={config.overlayColor}
                onChangeComplete={(color) => {
                  patchWallSettings({ overlayColor: color.toHexString() });
                }}
              />
            }
          />
        </SettingsSubGroup>
      </SettingsGroup>
    </>
  );
}
