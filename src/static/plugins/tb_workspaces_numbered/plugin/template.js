return Group({
  content: workspaces.map((w, idx) => {
    const isActive = w.id === activeWorkspace;

    return Button({
      content: idx + 1,
      style: {
        fontWeight: 600,
        color: isActive ? "var(--system-accent-color)" : "currentColor",
      },
      onClick: `invoke(SeelenCommand.SwitchWorkspace, { workspaceId: '${w.id}' })`,
    });
  }),
  style: {
    display: "flex",
    gap: "0.25rem",
  },
});
