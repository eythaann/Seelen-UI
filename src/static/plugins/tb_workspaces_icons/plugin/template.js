// Workspaces with the icons of their windows. Based on tb_workspaces_dotted:
// same active grid row, click-to-switch and active highlight, but each
// workspace pill shows up to MAX_ICONS icons of the windows placed on it.
// Empty workspaces fall back to the dotted-style dot.
//
// The Workspaces scope only carries raw window handles (`w.windows`), so the
// `Windows` scope is used to join handles with app details (exe path / umid /
// title) and render real application icons through `AppIcon`. Icons resolve
// through the icon pack system, so UWP apps (umid) are supported too.
//
// The pill tooltip lists the workspace name plus the titles of its windows.
const MAX_ICONS = 5;
const MAX_TOOLTIP_TITLES = 5;

const winByHwnd = {};
for (const win of windows) {
  winByHwnd[win.hwnd] = win;
}

const iconFor = (win) => {
  const path = win.process && win.process.path ? win.process.path : null;
  const umid = win.umid || null;
  return AppIcon({ path, umid });
};

const activeRow = workspaces.find((row) => row.some((w) => w.id === activeWorkspace));

if (!activeRow) {
  return null;
}

return Group({
  content: activeRow.map((w, idx) => {
    const isActive = w.id === activeWorkspace;
    const label = w.name || `Workspace ${idx + 1}`;
    const switchWs = `invoke(SeelenCommand.SwitchWorkspace, { workspaceId: '${w.id}' })`;

    const allWins = (w.windows || [])
      .map((hwnd) => winByHwnd[hwnd])
      .filter((win) => win && ((win.process && win.process.path) || win.umid));
    const wins = allWins.slice(0, MAX_ICONS);

    if (!allWins.length) {
      return Button({
        tooltip: `${label} (0)`,
        content: "",
        style: {
          padding: 0,
          width: "0.5rem",
          height: "0.5rem",
          borderRadius: "0.25rem",
          backgroundColor: isActive ? "var(--system-accent-color)" : "currentColor",
          opacity: isActive ? 1 : 0.5,
          transition: "backgroundColor 0.2s ease-in-out",
        },
        onClick: switchWs,
      });
    }

    const titles = allWins.map((win) => win.title).filter(Boolean);
    let tooltip = `${label} (${(w.windows || []).length})`;
    if (titles.length) {
      tooltip += "\n" + titles.slice(0, MAX_TOOLTIP_TITLES).join("\n");
      const rest = titles.length - MAX_TOOLTIP_TITLES;
      if (rest > 0) {
        tooltip += `\n+${rest} more`;
      }
    }

    const pillStyle = {
      display: "flex",
      alignItems: "center",
      gap: "0.15rem",
      padding: "0 0.35rem",
      height: "1.35rem",
      borderRadius: "0.7rem",
      transition: "backgroundColor 0.2s ease-in-out, boxShadow 0.2s ease-in-out",
    };
    if (isActive) {
      pillStyle.backgroundColor = "color-mix(in srgb, var(--system-accent-color) 22%, transparent)";
      pillStyle.boxShadow = "0 0 0 1.5px var(--system-accent-color)";
    }

    return Button({
      tooltip,
      content: wins.map(iconFor),
      style: pillStyle,
      onClick: switchWs,
    });
  }),
  style: {
    display: "flex",
    alignItems: "center",
    gap: "0.45rem",
    padding: "0 var(--item-padding)",
  },
});
