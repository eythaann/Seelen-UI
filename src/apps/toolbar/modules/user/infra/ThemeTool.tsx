// TODO(lurius): use this file on another module
import { ResourceMetadata } from '@seelen-ui/lib';
import { IconPack, Theme } from '@seelen-ui/lib/types';
import { Checkbox, Tooltip, Transfer } from 'antd';
import { TransferItem } from 'antd/es/transfer';
import { Reorder } from 'framer-motion';
import { t } from 'i18next';
import { useState } from 'react';

import { ResourceText } from 'src/apps/shared/components/ResourceText';

interface ThemeToolItem extends TransferItem {
  metadata: ResourceMetadata;
}
export interface ThemeToolArgs {
  dataSource: Theme[] | IconPack[];
  usingThemes: string[];
  setSelectedThemes: (themes: string[]) => void;
}

export function ThemeTool({ dataSource, usingThemes, setSelectedThemes }: ThemeToolArgs) {
  const [items] = useState<ThemeToolItem[]>(dataSource.map((value) => ({
    metadata: value.metadata,
    key: value.metadata.filename,
    title: ResourceText({ text: value.metadata.displayName }) ?? '',
    disabled: ('styles' in value && value.metadata.filename === 'default') || ('apps' in value && value.metadata.filename === 'system'),
  } as ThemeToolItem)));

  return (
    <Transfer
      dataSource={items}
      titles={[t('userhome.seelen_options.theme_selector.available'), t('userhome.seelen_options.theme_selector.selected')]}
      targetKeys={usingThemes}
      onChange={(selected, direction, movedKeys) => {
        if (direction === 'right') {
          setSelectedThemes([...usingThemes, ...(movedKeys as string[])]);
        } else {
          setSelectedThemes(selected as string[]);
        }
      }}
      className="userhome-quicksettings-themetool-transfer"
      showSelectAll={false}
    >
      {(props) => {
        const { dataSource, selectedKeys, onItemSelect, direction } = props;

        return (
          <Reorder.Group
            onReorder={(values) => {
              if (direction === 'right') {
                setSelectedThemes(values.map((theme) => theme.metadata.filename));
              }
            }}
            axis="y"
            values={props.dataSource}
          >
            {dataSource.map((theme) => {
              const key = theme.metadata.filename;
              return (
                <Tooltip
                  key={key}
                  placement={direction === 'right' ? 'left' : 'right'}
                  mouseEnterDelay={0.6}
                  color="#111" // make it solid
                  title={
                    <div>
                      <h2 className="userhome-quicksettings-themetool-title">
                        <ResourceText text={theme.metadata.displayName} />
                      </h2>
                      <p>
                        <b>{t('userhome.seelen_options.theme_selector.author')}: </b>
                        <ResourceText text={theme.metadata.author} />
                      </p>
                      <div className="userhome-quicksettings-themetool-tags">
                        <div>
                          <b>{t('userhome.seelen_options.theme_selector.tags')}:</b>
                        </div>
                        {theme.metadata.tags.map((tag) => (
                          <div key={tag} className="userhome-quicksettings-themetool-tag">
                            {tag}
                          </div>
                        ))}
                      </div>
                      <p>
                        <b>{t('userhome.seelen_options.theme_selector.description')}: </b>
                        <ResourceText text={theme.metadata.description} />
                      </p>
                    </div>
                  }
                >
                  <Reorder.Item value={theme} drag={direction === 'right' ? 'y' : false}>
                    <Checkbox
                      disabled={theme.disabled}
                      checked={selectedKeys.includes(key)}
                      onChange={(e) => onItemSelect(key, e.target.checked)}
                    >
                      <ResourceText text={theme.metadata.displayName} />
                    </Checkbox>
                  </Reorder.Item>
                </Tooltip>
              );
            })}
          </Reorder.Group>
        );
      }}
    </Transfer>
  );
}