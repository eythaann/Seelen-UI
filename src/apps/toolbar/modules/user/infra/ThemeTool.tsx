import { IconPack, Theme } from '@seelen-ui/lib/types';
import { Checkbox, Tooltip, Transfer } from 'antd';
import { TransferItem } from 'antd/es/transfer';
import { Reorder } from 'framer-motion';
import { t } from 'i18next';
import { useState } from 'react';

interface ThemeToolItem extends TransferItem {
  info: {
    displayName: string;
    author: string;
    description: string;
    filename: string;
    tags: Array<string>;
  };
}
export interface ThemeToolArgs {
  dataSource: Theme[] | IconPack[];
  usingThemes: string[];
  setSelectedThemes: (themes: string[]) => void;
}

export function ThemeTool({ dataSource, usingThemes, setSelectedThemes }: ThemeToolArgs) {
  const [items] = useState<ThemeToolItem[]>(dataSource.map((value) => ({
    info: {
      displayName: value.info.displayName,
      author: value.info.author,
      description: value.info.description,
      filename: value.info.filename,
      tags: value.info.tags,
    },
    key: value.info.filename,
    title: value.info.displayName,
    disabled: ('styles' in value && value.info.filename === 'default') || ('apps' in value && value.info.filename === 'system'),
  })));

  return (
    <Transfer
      dataSource={items}
      titles={[t('general.theme.available'), t('general.theme.selected')]}
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
                setSelectedThemes(values.map((theme) => theme.info.filename));
              }
            }}
            axis="y"
            values={props.dataSource}
          >
            {dataSource.map((theme) => {
              const key = theme.info.filename;
              return (
                <Tooltip
                  key={key}
                  placement={direction === 'right' ? 'left' : 'right'}
                  mouseEnterDelay={0.6}
                  color="#111" // make it solid
                  title={
                    <div>
                      <h2 className="userhome-quicksettings-themetool-title">{theme.info.displayName}</h2>
                      <p>
                        <b>{t('general.resource.author')}: </b>
                        {theme.info.author}
                      </p>
                      <div className="userhome-quicksettings-themetool-tags">
                        <div>
                          <b>{t('general.resource.tags')}:</b>
                        </div>
                        {theme.info.tags.map((tag) => (
                          <div key={tag} className="userhome-quicksettings-themetool-tag">
                            {tag}
                          </div>
                        ))}
                      </div>
                      <p>
                        <b>{t('general.resource.description')}: </b>
                        {theme.info.description}
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
                      {theme.info.displayName}
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