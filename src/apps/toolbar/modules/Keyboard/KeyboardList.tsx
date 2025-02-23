import { invoke, SeelenCommand } from '@seelen-ui/lib';
import { PropsWithChildren, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSelector } from 'react-redux';

import { BackgroundByLayersV2 } from 'src/apps/seelenweg/components/BackgroundByLayers/infra';

import { Selectors } from '../shared/store/app';
import { AnimatedPopover } from 'src/apps/shared/components/AnimatedWrappers';
import { Icon } from 'src/apps/shared/components/Icon';
import { useWindowFocusChange } from 'src/apps/shared/hooks';
import { cx } from 'src/apps/shared/styles';

function KeyboardSelector() {
  const languages = useSelector(Selectors.languages);

  const { t } = useTranslation();

  return (
    <BackgroundByLayersV2 className="keyboard-selector">
      <div className="keyboard-selector-header">{t('keyboard.title')}</div>
      <div className="keyboard-selector-body">
        {languages.map((lang) => {
          return lang.inputMethods.map((keyboard) => {
            return (
              <button
                key={`${lang.code}-${keyboard.id}`}
                className={cx('keyboard-selector-entry', {
                  'keyboard-selector-entry-active': keyboard.active,
                })}
                onClick={() => {
                  invoke(SeelenCommand.SystemSetKeyboardLayout, {
                    id: keyboard.id,
                    handle: keyboard.handle,
                  });
                }}
              >
                <div className="keyboard-selector-entry-icon">
                  <Icon iconName="FaRegKeyboard" />
                </div>
                <div className="keyboard-selector-entry-info">
                  <span className="keyboard-selector-entry-lang">{lang.name}</span>
                  <span className="keyboard-selector-entry-keyboard">{keyboard.displayName}</span>
                </div>
              </button>
            );
          });
        })}
      </div>
      <div className="keyboard-selector-footer">
        <button
          className="keyboard-selector-footer-button"
          onClick={() => invoke(SeelenCommand.OpenFile, { path: 'ms-settings:keyboard' })}
        >
          {t('keyboard.more')}
        </button>
      </div>
    </BackgroundByLayersV2>
  );
}

export function WithKeyboardSelector({ children }: PropsWithChildren) {
  const [openPreview, setOpenPreview] = useState(false);

  useWindowFocusChange((focused) => {
    if (!focused) {
      setOpenPreview(false);
    }
  });

  return (
    <AnimatedPopover
      animationDescription={{
        openAnimationName: 'keyboard-selector-open',
        closeAnimationName: 'keyboard-selector-close',
      }}
      open={openPreview}
      trigger="click"
      onOpenChange={setOpenPreview}
      arrow={false}
      content={<KeyboardSelector />}
    >
      {children}
    </AnimatedPopover>
  );
}
