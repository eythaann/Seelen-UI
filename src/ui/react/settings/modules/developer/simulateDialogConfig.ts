import type { Dialog } from "@seelen-ui/lib/types";

export function createSampleDialog(): Dialog {
  return {
    identifier: crypto.randomUUID(),
    width: 400,
    height: 200,
    title: [{ type: "text", value: "Test Dialog", styles: null }],
    content: [
      {
        type: "text",
        value: "This is a test dialog triggered from DevTools.",
        styles: null,
      },
    ],
    footer: [
      {
        type: "button",
        inner: [{ type: "text", value: "Close", styles: null }],
        onClick: "exit",
        styles: null,
      },
    ],
  };
}
