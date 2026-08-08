import { invoke, SeelenCommand, SeelenEvent, subscribe } from "@seelen-ui/lib";
import type { UserAppWindow } from "@seelen-ui/lib/types";
import { Button, Empty, Input, Spin, Tooltip } from "antd";
import { useCallback, useEffect, useMemo, useState } from "react";

import { FileIcon, Icon } from "libs/ui/react/components/Icon";
import { SettingsGroup, SettingsSubGroup } from "../../../components/SettingsBox/index.tsx";
import { useTranslation } from "react-i18next";
import cs from "./TaskSwitcherWindowPicker.module.css";

interface Props {
  mode: string;
  patterns: string;
  onChange: (patterns: string) => void;
}

interface AppEntry {
  key: string;
  name: string;
  executable: string;
  path: string | null;
  umid: string | null;
  windows: number;
}

function executableName(path: string | null) {
  return path?.split(/[\\/]/).pop() || "";
}

function makeEntries(windows: UserAppWindow[]): AppEntry[] {
  const entries = new Map<string, AppEntry>();
  for (const window of windows) {
    const path = window.process.path;
    const executable = executableName(path);
    const key = (path || executable || window.appName || window.title).toLowerCase();
    const current = entries.get(key);
    if (current) {
      current.windows += 1;
      continue;
    }
    entries.set(key, {
      key,
      name: window.appName || executable || window.title,
      executable,
      path,
      umid: window.umid,
      windows: 1,
    });
  }
  return [...entries.values()].sort((a, b) => a.name.localeCompare(b.name));
}

function parsePatterns(patterns: string) {
  return patterns.split(/\r?\n/).map((pattern) => pattern.trim()).filter(Boolean);
}

function matchingPatterns(entry: AppEntry, patterns: string[]) {
  const values = [entry.name, entry.executable, entry.path]
    .filter((value): value is string => !!value)
    .map((value) => value.toLowerCase());
  return patterns.filter((pattern) => values.some((value) => value.includes(pattern.toLowerCase())));
}

export function TaskSwitcherWindowPicker({ mode, patterns, onChange }: Props) {
  const { t } = useTranslation();
  const [windows, setWindows] = useState<UserAppWindow[]>([]);
  const [search, setSearch] = useState("");
  const [loading, setLoading] = useState(true);
  const [loadFailed, setLoadFailed] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    setLoadFailed(false);
    try {
      setWindows(await invoke(SeelenCommand.GetUserAppWindows));
    } catch {
      setLoadFailed(true);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let active = true;
    void refresh();
    let unsubscribe: (() => void) | undefined;
    void subscribe(SeelenEvent.UserAppWindowsChanged, ({ payload }) => {
      if (active) {
        setWindows(payload);
        setLoading(false);
        setLoadFailed(false);
      }
    }).then((stop) => {
      if (active) {
        unsubscribe = stop;
      } else {
        stop();
      }
    });
    return () => {
      active = false;
      unsubscribe?.();
    };
  }, [refresh]);

  const entries = useMemo(() => makeEntries(windows), [windows]);
  const visibleEntries = useMemo(() => {
    const query = search.trim().toLowerCase();
    if (!query) return entries;
    return entries.filter((entry) =>
      [entry.name, entry.executable, entry.path].some((value) => value?.toLowerCase().includes(query))
    );
  }, [entries, search]);

  if (mode !== "blacklist" && mode !== "whitelist") return null;

  const current = parsePatterns(patterns);

  const toggle = (entry: AppEntry) => {
    const matched = new Set(matchingPatterns(entry, current));
    const next = matched.size > 0
      ? current.filter((pattern) => !matched.has(pattern))
      : [...current, entry.executable || entry.name];
    onChange(next.join("\n"));
  };

  return (
    <SettingsGroup>
      <SettingsSubGroup label={t("task_switcher.application_list.title")}>
        <div className={cs.header}>
          <span className={cs.description}>{t("task_switcher.application_list.description")}</span>
          <Button
            type="text"
            title={t("task_switcher.application_list.refresh")}
            aria-label={t("task_switcher.application_list.refresh")}
            icon={<Icon iconName="RiRefreshLine" />}
            loading={loading}
            onClick={() => void refresh()}
          />
        </div>
        <Input.Search
          allowClear
          value={search}
          placeholder={t("task_switcher.application_list.search")}
          onChange={(event) => setSearch(event.currentTarget.value)}
        />
        <div className={cs.list}>
          {loading && windows.length === 0
            ? <div className={cs.status}><Spin size="small" /></div>
            : visibleEntries.map((entry) => {
            const included = matchingPatterns(entry, current).length > 0;
            return (
              <div className={cs.row} key={entry.key}>
                <FileIcon path={entry.path} umid={entry.umid} className={cs.icon} />
                <div className={cs.info}>
                  <div className={cs.name}>{entry.name}</div>
                  <div className={cs.meta}>
                    {entry.executable || entry.path || t("task_switcher.application_list.no_path")}
                    <span>{t("task_switcher.application_list.windows", { count: entry.windows })}</span>
                  </div>
                </div>
                <Tooltip title={included ? t("task_switcher.application_list.remove") : t("task_switcher.application_list.add")}>
                  <Button
                    type={included ? "primary" : "default"}
                    shape="circle"
                    aria-label={included ? t("task_switcher.application_list.remove") : t("task_switcher.application_list.add")}
                    icon={<Icon iconName={included ? "RiCheckLine" : "RiAddLine"} />}
                    onClick={() => toggle(entry)}
                  />
                </Tooltip>
              </div>
            );
          })}
          {!loading && visibleEntries.length === 0 && (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={t(loadFailed
                ? "task_switcher.application_list.load_failed"
                : "task_switcher.application_list.empty")}
            />
          )}
        </div>
      </SettingsSubGroup>
    </SettingsGroup>
  );
}
