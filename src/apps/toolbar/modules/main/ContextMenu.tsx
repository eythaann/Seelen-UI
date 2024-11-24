import { Menu, Popover } from 'antd';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from 'src/apps/seelenweg/components/BackgroundByLayers/infra';

import { RootActions, Selectors } from '../shared/store/app';
import { SavePlaceholderAsCustom } from './application';

export function MainContextMenu() {
  const plugins = useSelector(Selectors.plugins);

  const dispatch = useDispatch();
  const { t } = useTranslation();

  return (
    <BackgroundByLayersV2 className="tb-context-menu-container">
      <Menu
        className="tb-context-menu"
        items={[
          {
            key: 'add_module',
            label: (
              <Popover
                trigger={['hover']}
                placement="rightBottom"
                arrow={false}
                content={
                  <BackgroundByLayersV2 className="tb-context-menu-container">
                    <Menu
                      className="tb-context-menu"
                      items={plugins.map((p) => ({
                        key: p.id,
                        label: p.id,
                        onClick: () => {
                          dispatch(RootActions.addItem(p.id));
                          SavePlaceholderAsCustom();
                        },
                      }))}
                    />
                  </BackgroundByLayersV2>
                }
              >
                <div>{t('context_menu.add_module')}</div>
              </Popover>
            ),
          },
        ]}
      />
    </BackgroundByLayersV2>
  );
}
