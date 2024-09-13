import { Checkbox, Tooltip, Transfer } from 'antd';
import { Reorder } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { RootActions } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';

import cs from './index.module.css';

export function Themes() {
  const themes = useSelector(RootSelectors.availableThemes);
  const usingThemes = useSelector(RootSelectors.selectedThemes);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const dataSource = themes.map((theme) => ({
    ...theme,
    key: theme.info.filename,
    title: theme.info.displayName,
    disabled: theme.info.filename === 'default',
  }));

  return (
    <Transfer
      dataSource={dataSource}
      titles={[t('general.theme.available'), t('general.theme.selected')]}
      targetKeys={usingThemes}
      onChange={(selected) => {
        dispatch(RootActions.setSelectedThemes(selected as string[]));
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
                      <h2 className={cs.title}>{theme.info.displayName}</h2>
                      <p>
                        <b>{t('general.theme.author')}: </b>
                        {theme.info.author}
                      </p>
                      <div className={cs.tagList}>
                        <div>
                          <b>{t('general.theme.tags')}:</b>
                        </div>
                        {theme.info.tags.map((tag) => (
                          <div key={tag} className={cs.tag}>
                            {tag}
                          </div>
                        ))}
                      </div>
                      <p>
                        <b>{t('general.theme.description')}: </b>
                        {theme.info.description}
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
