import { Checkbox, Tooltip, Transfer } from 'antd';
import { Reorder } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { RootActions } from '../../../shared/store/app/reducer';
import { RootSelectors } from '../../../shared/store/app/selectors';
import { ResourceText } from 'src/apps/shared/components/ResourceText';

import cs from './index.module.css';

export function IconPacks() {
  const iconPacks = useSelector(RootSelectors.availableIconPacks);
  const usingIcons = useSelector(RootSelectors.iconPacks).filter((x) =>
    iconPacks.some((y) => y.metadata.filename === x),
  );

  const dispatch = useDispatch();
  const { t } = useTranslation();

  const dataSource = iconPacks.map((pack) => ({
    ...pack,
    key: pack.metadata.filename,
  }));

  return (
    <Transfer
      dataSource={dataSource}
      titles={[t('general.icon_pack.available'), t('general.icon_pack.selected')]}
      targetKeys={usingIcons}
      onChange={(selected, direction, movedKeys) => {
        if (direction === 'right') {
          dispatch(RootActions.setIconPacks([...usingIcons, ...(movedKeys as string[])]));
        } else {
          dispatch(RootActions.setIconPacks(selected as string[]));
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
                dispatch(
                  RootActions.setIconPacks(values.map((iconPack) => iconPack.metadata.filename)),
                );
              }
            }}
            axis="y"
            values={props.dataSource}
          >
            {dataSource.map((pack) => {
              const key = pack.metadata.filename;
              return (
                <Tooltip
                  key={key}
                  placement={direction === 'right' ? 'left' : 'right'}
                  mouseEnterDelay={0.6}
                  color="#111" // make it solid
                  title={
                    <div>
                      <h2 className={cs.title}>
                        <ResourceText text={pack.metadata.displayName} />
                      </h2>
                      <div className={cs.tagList}>
                        <div>
                          <b>{t('general.resource.tags')}:</b>
                        </div>
                        {pack.metadata.tags.map((tag) => (
                          <div key={tag} className={cs.tag}>
                            {tag}
                          </div>
                        ))}
                      </div>
                      <p>
                        <b>{t('general.resource.description')}: </b>
                        <ResourceText text={pack.metadata.description} />
                      </p>
                    </div>
                  }
                >
                  <Reorder.Item value={pack} drag={direction === 'right' ? 'y' : false}>
                    <Checkbox
                      disabled={pack.metadata.filename === 'system'}
                      checked={selectedKeys.includes(key)}
                      onChange={(e) => onItemSelect(key, e.target.checked)}
                    >
                      <ResourceText text={pack.metadata.displayName} />
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
