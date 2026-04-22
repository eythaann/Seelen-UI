import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { memo, useCallback } from "preact/compat";
import { useTranslation } from "react-i18next";

export const ShowDesktopButton = memo(function ShowDesktopButton() {
  const { t } = useTranslation();

  const handleClick = useCallback(() => {
    invoke(SeelenCommand.ShowDesktop);
  }, []);

  return <button className="ft-corner-button" onClick={handleClick} title={t("show_desktop")} />;
});
