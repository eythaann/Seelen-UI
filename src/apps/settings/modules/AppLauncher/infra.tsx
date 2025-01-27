import { SeelenLauncherMonitor } from '@seelen-ui/lib';
import { SeelenLauncherRunner } from '@seelen-ui/lib/types';
import { Button, Input, Select, Switch } from 'antd';
import { Reorder } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { useDispatch, useSelector } from 'react-redux';

import { newSelectors, RootActions } from '../shared/store/app/reducer';
import { OptionsFromEnum } from '../shared/utils/app';
import { Icon } from 'src/apps/shared/components/Icon';

import { SettingsGroup, SettingsOption } from '../../components/SettingsBox';
import cs from './index.module.css';

export function AppLauncherSettings() {
  const launcher = useSelector(newSelectors.launcher);
  const { enabled, monitor, runners } = launcher;

  const dispatch = useDispatch();
  const { t } = useTranslation();

  function onChangeEnabled(value: boolean) {
    dispatch(RootActions.setLauncher({ ...launcher, enabled: value }));
  }

  function onChangeMonitor(value: SeelenLauncherMonitor) {
    dispatch(RootActions.setLauncher({ ...launcher, monitor: value }));
  }

  function onChangeRunners(runners: SeelenLauncherRunner[]) {
    dispatch(RootActions.setLauncher({ ...launcher, runners }));
  }

  function onAddRunner() {
    onChangeRunners([...runners, {
      id: crypto.randomUUID(),
      label: '',
      program: '',
      readonly: false,
    }]);
  }

  function onRemoveRunner(idx: number) {
    let newRunners = [...runners];
    newRunners.splice(idx, 1);
    onChangeRunners(newRunners);
  }

  function onChangeRunnerLabel(idx: number, value: string) {
    let runner = runners[idx];
    if (!runner) return;
    let newRunners = [...runners];
    newRunners[idx] = { ...runner, label: value };
    onChangeRunners(newRunners);
  }

  function onChangeRunnerProgram(idx: number, value: string) {
    let runner = runners[idx];
    if (!runner) return;
    let newRunners = [...runners];
    newRunners[idx] = { ...runner, program: value };
    onChangeRunners(newRunners);
  }

  return (
    <>
      <SettingsGroup>
        <SettingsOption>
          <b>{t('app_launcher.enable')}</b>
          <Switch value={enabled} onChange={onChangeEnabled} />
        </SettingsOption>
        <SettingsOption>
          <b>{t('app_launcher.monitor')}</b>
          <Select
            options={OptionsFromEnum(t, SeelenLauncherMonitor, 'app_launcher.launch_on')}
            value={monitor}
            onChange={onChangeMonitor}
          />
        </SettingsOption>
      </SettingsGroup>

      <SettingsGroup>
        <b>{t('app_launcher.runners.label')}</b>
        <Reorder.Group
          values={runners}
          onReorder={onChangeRunners}
          className={cs.runnerList}
          axis="y"
        >
          {runners.map((runner, idx) => (
            <Reorder.Item key={runner.id} value={runner} className={cs.runner}>
              <Input
                value={
                  runner.label.startsWith('t:') ? t(runner.label.replace('t:', '')) : runner.label
                }
                placeholder="-"
                disabled={runner.readonly}
                onChange={(e) => onChangeRunnerLabel(idx, e.target.value)}
              />
              <Input
                value={runner.program}
                placeholder="C:\...\program.exe"
                disabled={runner.readonly}
                onChange={(e) => onChangeRunnerProgram(idx, e.target.value)}
              />
              <Button type="primary" onClick={() => onRemoveRunner(idx)} disabled={runner.readonly}>
                <Icon iconName="IoTrash" size={14} />
              </Button>
            </Reorder.Item>
          ))}
          <Button type="primary" className={cs.runnerAdd} onClick={onAddRunner}>
            <Icon iconName="MdLibraryAdd" size={14} />
          </Button>
        </Reorder.Group>
      </SettingsGroup>
    </>
  );
}
