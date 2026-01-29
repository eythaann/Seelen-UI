return Group({
  content: workspaces.map((w) => {
    const isActive = w.id === activeWorkspace;

    return Button({
      content: "",
      style: {
        padding: 0,
        width: isActive ? "1.5rem" : "0.5rem",
        height: "0.5rem",
        borderRadius: "0.25rem",
        backgroundColor: isActive ? "var(--system-accent-color)" : "currentColor",
        transition: "width 0.2s ease-in-out, backgroundColor 0.2s ease-in-out",
      },
      onClick: `invoke(SeelenCommand.SwitchWorkspace, { workspaceId: '${w.id}' })`,
    });
  }),
  style: {
    display: "flex",
    gap: "0.75rem",
  },
});
