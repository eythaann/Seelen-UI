import * as logger from "./_ConsoleWrapper.ts";

function StringifyParams(params: any[]): string {
  return params.reduce((acc, current) => {
    if (typeof current === "string") {
      return acc + " " + current;
    }

    if (current instanceof Error) {
      return `${acc} ${current.name}(${current.message})\n${current.stack}`;
    }

    if (typeof current === "object") {
      let stringObj = "";
      try {
        stringObj = JSON.stringify(current, null, 2);
      } catch (_e) {
        stringObj = `${current}`;
      }
      return acc + " " + stringObj;
    }

    return acc + " " + `${current}`;
  }, "");
}

function forwardConsole(
  fnName: "log" | "trace" | "debug" | "info" | "warn" | "error",
  logger: (message: string) => Promise<void>,
) {
  const original = console[fnName];
  console[fnName] = (...params: any[]) => {
    original(...params);
    let message = StringifyParams(params);
    /// ignore Ant Design Warnings
    if (message.includes("[Ant Design CSS-in-JS]")) {
      return;
    }
    logger(message);
  };
}

forwardConsole("trace", logger.trace);
forwardConsole("debug", logger.debug);
forwardConsole("info", logger.info);
forwardConsole("warn", logger.warn);
forwardConsole("error", logger.error);

globalThis.addEventListener("unhandledrejection", (event) => {
  console.error("Unhandled Rejection", event.reason);
});

globalThis.addEventListener(
  "error",
  (event) => {
    // could be undefined on fetch errors
    if (event.error || event.message) {
      console.error("Uncaught Error", event.error || event.message);
    }
  },
  true,
);
