import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';

import cs from './infra.module.css';

export function Information() {
  return (
    <div className={cs.info}>
      <SettingsGroup>
        <SettingsSubGroup label="Documentation">
          <SettingsOption>
            <span>komorebi:</span>
            <a href="https://lgug2z.github.io/komorebi" target="_blank">
              lgug2z.github.io/komorebi
            </a>
          </SettingsOption>
          <SettingsOption>
            <span>komorebi-ui:</span>
            <a href="https://github.com/eythaann/komorebi-ui" target="_blank">
              github.com/eythaann/komorebi-ui
            </a>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>

      <SettingsGroup>
        <SettingsSubGroup label="Follow me:">
          <SettingsOption>
            <span>Github:</span>
            <a href="https://github.com/eythaann" target="_blank">
              github.com/eythaann
            </a>
          </SettingsOption>
        </SettingsSubGroup>
      </SettingsGroup>
    </div>
  );
}
