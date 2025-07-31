import { invoke, SeelenCommand, Widget } from '@seelen-ui/lib';
import { Button } from 'antd';

import { SettingsGroup } from '../../components/SettingsBox';

export function Shortcuts() {
  return (
    <SettingsGroup>
      <Button
        onClick={() => {
          invoke(SeelenCommand.RequestToUserInputShortcut, { callbackEvent: 'finished' });
          Widget.getCurrent().webview.once('finished', (e) => {
            console.debug('It works! nice.', e.payload);
          });
        }}
      >
        Click Me
      </Button>
    </SettingsGroup>
  );
}
