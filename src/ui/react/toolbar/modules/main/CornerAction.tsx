import { invoke, SeelenCommand } from "@seelen-ui/lib";
import { memo, useCallback } from "preact/compat";

export const ShowDesktopButton = memo(function ShowDesktopButton() {
  const handleClick = useCallback(() => {
    invoke(SeelenCommand.ShowDesktop);
  }, []);

  return (
    <button
      className="ft-corner-button"
      onClick={handleClick}
      title="Show Desktop"
      aria-label="Show Desktop"
    />
  );
});
