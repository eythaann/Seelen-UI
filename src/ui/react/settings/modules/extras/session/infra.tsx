import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { Avatar, Button, Tooltip } from "antd";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { session } from "../../../state/session.ts";
import cs from "./infra.module.css";
import { cx } from "libs/ui/react/utils/styling.ts";

const SKY_UPGRADE_URL = "https://seelen.io/sky";

export function SessionView() {
  const currentSession = session.value;

  if (currentSession) {
    return <SessionProfile />;
  }

  return <LoginForm />;
}

function SessionProfile() {
  const currentSession = session.value!;

  const { t } = useTranslation();
  const [loggingOut, setLoggingOut] = useState(false);

  async function handleLogout() {
    setLoggingOut(true);
    try {
      await invoke(SeelenCommand.SeelenLogout);
    } finally {
      setLoggingOut(false);
    }
  }

  const displayName = currentSession.displayName || currentSession.username;
  const avatarLetter = displayName[0]?.toUpperCase() ?? "?";
  const isFree = currentSession.plan === "Free";

  return (
    <div className={cs.profileCard}>
      <div className={cs.profileHeader}>
        <Avatar className={cs.avatar} size={64} src={currentSession.avatar || undefined}>
          {!currentSession.avatar && avatarLetter}
        </Avatar>

        <div className={cs.profileInfo}>
          <span className={cs.displayName}>
            {displayName}{" "}
            <div className={cx(cs.planTag, { [cs.skyPlanTag]: !isFree })}>
              {currentSession.plan}
            </div>
          </span>

          <span className={cs.username}>@{currentSession.username}</span>
          {currentSession.email && <span className={cs.email}>{currentSession.email}</span>}
        </div>
      </div>

      {isFree && (
        <a className={cs.upgradeBtn} href={SKY_UPGRADE_URL} target="_blank">
          {t("session.upgrade_to")} Sky
        </a>
      )}

      <div className={cs.profileActions}>
        <Button
          className={cs.logoutBtn}
          type="text"
          danger
          loading={loggingOut}
          onClick={handleLogout}
        >
          {t("session.logout")}
        </Button>
      </div>
    </div>
  );
}

function LoginForm() {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleLogin() {
    setLoading(true);
    setError(null);
    try {
      await invoke(SeelenCommand.SeelenLogin);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className={cs.loginCard}>
      <img src="./company_logo.svg" className={cs.logo} alt="Seelen UI" />
      <h3 className={cs.loginTitle}>Seelen Corp.</h3>

      {error && <p className={cs.error}>{error}</p>}

      <Tooltip placement="bottom" title={t("session.sign_in_hint")}>
        <Button type="primary" loading={loading} onClick={handleLogin} className={cs.loginBtn}>
          {t("session.sign_in")}
        </Button>
      </Tooltip>
    </div>
  );
}
