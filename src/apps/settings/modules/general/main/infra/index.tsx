import { Icon } from '../../../../../shared/components/Icon';
import { Theme } from '../../../../../shared/schemas/Theme';
import {
  SettingsGroup,
  SettingsOption,
  SettingsSubGroup,
} from '../../../../components/SettingsBox';
import { Button, Flex, Select, Switch, Tag, Tooltip } from 'antd';
import { Reorder } from 'framer-motion';
import { useState } from 'react';
import { useSelector } from 'react-redux';

import { startup } from '../../../shared/tauri/infra';
import { useAppDispatch } from '../../../shared/utils/infra';

import { RootActions } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';

import { TAGS_COLORS } from '../domain';

import cs from './index.module.css';

export function General() {
  const autostartStatus = useSelector(RootSelectors.autostart);
  const themes = useSelector(RootSelectors.availableThemes);
  const usingThemes = useSelector(RootSelectors.selectedTheme);

  const [selectedThemeStr, setSelectedThemeStr] = useState<string | null>(null);

  const dispatch = useAppDispatch();

  const onAutoStart = async (value: boolean) => {
    if (value) {
      await startup.enable();
    } else {
      await startup.disable();
    }
    dispatch(RootActions.setAutostart(value));
  };

  const selectedTheme = themes.find((theme) => theme.info.filename === selectedThemeStr);
  const selectedThemeIsAdded = !!selectedThemeStr && usingThemes.includes(selectedThemeStr);

  const themesById = themes.reduce((acc, theme) => {
    acc[theme.info.filename] = theme;
    return acc;
  }, {} as Record<string, Theme>);

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <span style={{ fontWeight: 600 }}>Run Seelen UI at startup?</span>
          <Switch onChange={onAutoStart} value={autostartStatus} />
        </SettingsOption>
      </SettingsGroup>
      <SettingsGroup>
        <SettingsSubGroup
          label={
            <SettingsOption>
              <b>Theme info: </b>
              <Select
                style={{ width: '200px' }}
                value={selectedThemeStr}
                allowClear
                options={themes.map((theme) => ({
                  label: theme.info.displayName,
                  value: theme.info.filename,
                }))}
                onSelect={setSelectedThemeStr}
                onClear={() => setSelectedThemeStr(null)}
                placeholder="Select theme"
              />
            </SettingsOption>
          }
        >
          {selectedTheme && (
            <SettingsOption>
              <Flex gap="2px 0" wrap="wrap">
                <b style={{ marginRight: '4px' }}>Tags:</b>
                {selectedTheme.info.tags.map((tag, idx) => (
                  <Tag key={tag} color={TAGS_COLORS[idx % TAGS_COLORS.length]} bordered={false}>
                    {tag}
                  </Tag>
                ))}
              </Flex>
            </SettingsOption>
          )}
          {selectedTheme && (
            <SettingsOption>
              <b>Add to layers</b>
              <Tooltip title={selectedThemeIsAdded ? 'Already added' : ''}>
                <Button
                  type="dashed"
                  disabled={selectedThemeIsAdded}
                  style={{ width: '50px' }}
                  onClick={() => {
                    if (selectedThemeStr) {
                      dispatch(RootActions.setSelectedTheme([...usingThemes, selectedThemeStr]));
                    }
                  }}
                >
                  <b>+</b>
                </Button>
              </Tooltip>
            </SettingsOption>
          )}
          {selectedTheme && (
            <SettingsOption>
              <p>
                <b>Author: </b>
                {selectedTheme.info.author}
              </p>
            </SettingsOption>
          )}
          {selectedTheme && (
            <SettingsOption>
              <p>
                <b>Description: </b>
                {selectedTheme.info.description}
              </p>
            </SettingsOption>
          )}
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <b>Themes enabled:</b>
        <div>
          <Reorder.Group
            onReorder={(values) => dispatch(RootActions.setSelectedTheme(values))}
            values={usingThemes}
            axis="y"
            className={cs.resourceList}
          >
            {usingThemes.map((themeStr) => {
              const theme = themesById[themeStr];

              if (!theme) {
                return null;
              }

              return (
                <Reorder.Item key={theme.info.filename} value={themeStr} className={cs.resource}>
                  {theme.info.displayName}
                  <Button
                    type="text"
                    danger
                    onClick={() => dispatch(RootActions.removeTheme(themeStr))}
                    disabled={theme.info.filename === 'default'}
                  >
                    <Icon iconName="IoTrash" />
                  </Button>
                </Reorder.Item>
              );
            })}
          </Reorder.Group>
        </div>
      </SettingsGroup>
    </>
  );
}
