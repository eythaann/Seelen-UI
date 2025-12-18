import { invoke, SeelenCommand } from "@seelen-ui/lib";

export function ShowDesktopButton() {
  return (
    <button
      className="ft-corner-button"
      onClick={() => {
        invoke(SeelenCommand.ShowDesktop);
      }}
      title="Show Desktop"
      aria-label="Show Desktop"
    />
  );
}
