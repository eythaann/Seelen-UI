import type { Dialog } from "@seelen-ui/lib/types";

export function createSampleDialog(): Dialog {
  const identifier = crypto.randomUUID();
  return {
    identifier,
    width: 500,
    height: 200,
    title: [{ type: "text", value: "Test Dialog" }],
    content: [
      {
        type: "text",
        value: "This is a test dialog triggered from DevTools.",
      },
      {
        type: "text",
        value: `Identifier: ${identifier}`,
      },
    ],
    footer: [
      {
        type: "button",
        inner: [{ type: "text", value: "Close" }],
        onClick: "exit",
      },
    ],
  };
}
