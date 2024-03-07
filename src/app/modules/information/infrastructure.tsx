import { SettingsGroup, SettingsOption, SettingsSubGroup } from '../../components/SettingsBox';
import { Button, message, Upload } from 'antd';

import { LoadSettingsToStore, SaveStore } from '../shared/infrastructure/store';
import cs from './infra.module.css';

import { EnvConfig } from '../shared/domain/envConfig';

export function Information() {
  return (
    <div className={cs.info}>
      <SettingsGroup>
        <SettingsSubGroup label="Documentation">
          <SettingsOption>
            <span>komorebi <span className={cs.version}>v{EnvConfig.komorebiVersion}</span>:</span>
            <a href="https://lgug2z.github.io/komorebi" target="_blank">
              lgug2z.github.io/komorebi
            </a>
          </SettingsOption>
          <SettingsOption>
            <span>komorebi-ui <span className={cs.version}>v{EnvConfig.version}</span>:</span>
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

      <SettingsGroup>
        <SettingsOption>
          <span>Force Restart</span>
          <Button
            type="dashed"
            onClick={() => {
              window.backgroundApi.forceRestart();
            }}
          >
            ⟳
          </Button>
        </SettingsOption>
        <SettingsOption>
          <span>Load config file (will replace current configurations):</span>
          <Upload
            fileList={[]}
            onChange={async ({ file }) => {
              if (file.originFileObj?.path) {
                LoadSettingsToStore(file.originFileObj?.path)
                  .then(() => {
                    message.success('File load completed.');
                    SaveStore();
                  })
                  .catch((_e) => message.error('Error loading the file.'));
              }
            }}
            maxCount={1}
            beforeUpload={(file) => {
              const isJson = file.type === 'application/json';
              if (!isJson) {
                message.error(`${file.name} is not a json file`);
              }
              return isJson || Upload.LIST_IGNORE;
            }}
          >
            <Button>Select File</Button>
          </Upload>
        </SettingsOption>
      </SettingsGroup>
    </div>
  );
}
