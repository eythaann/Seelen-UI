import { Theme } from '@seelen-ui/lib/types';
import { Checkbox, Tooltip, Transfer } from 'antd';
import { Reorder } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { RootActions } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';
import { ResourceText } from 'src/apps/shared/components/ResourceText';

import cs from './index.module.css';

export function Themes() {
  const themes = useSelector(RootSelectors.availableThemes);
  const usingThemes = useSelector(RootSelectors.selectedThemes).filter((x) =>
    themes.some((y) => y.metadata.filename === x),
  );

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const dataSource = themes.map((theme) => ({
    ...theme,
    key: theme.metadata.filename,
    title: theme.metadata.displayName,
    disabled: theme.metadata.filename === 'default',
  }));

  return (
    <Transfer
      dataSource={dataSource as any}
      titles={[t('general.theme.available'), t('general.theme.selected')]}
      targetKeys={usingThemes}
      onChange={(selected, direction, movedKeys) => {
        if (direction === 'right') {
          dispatch(RootActions.setSelectedThemes([...usingThemes, ...(movedKeys as string[])]));
        } else {
          dispatch(RootActions.setSelectedThemes(selected as string[]));
        }
      }}
      className={cs.transfer}
      showSelectAll={false}
    >
      {(props) => {
        const { dataSource, selectedKeys, onItemSelect, direction } = props;

        return (
          <Reorder.Group
            onReorder={(values) => {
              if (direction === 'right') {
                dispatch(RootActions.setSelectedThemes(values.map((theme) => theme.info.filename)));
              }
            }}
            axis="y"
            values={props.dataSource}
          >
            {dataSource.map((_theme) => {
              const theme = _theme as Theme;
              const key = theme.metadata.filename;
              return (
                <Tooltip
                  key={key}
                  placement={direction === 'right' ? 'left' : 'right'}
                  mouseEnterDelay={0.6}
                  color="#111" // make it solid
                  title={
                    <div>
                      <h2 className={cs.title}>
                        <ResourceText text={theme.metadata.displayName} />
                      </h2>
                      <div className={cs.tagList}>
                        <div>
                          <b>{t('general.resource.tags')}:</b>
                        </div>
                        {theme.metadata.tags.map((tag: string) => (
                          <div key={tag} className={cs.tag}>
                            {tag}
                          </div>
                        ))}
                      </div>
                      <p>
                        <b>{t('general.resource.description')}: </b>
                        <ResourceText text={theme.metadata.description} />
                      </p>
                    </div>
                  }
                >
                  <Reorder.Item value={theme} drag={direction === 'right' ? 'y' : false}>
                    <Checkbox
                      disabled={key === 'default'}
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
