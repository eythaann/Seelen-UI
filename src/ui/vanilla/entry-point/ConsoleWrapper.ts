import * as logger from "./_ConsoleWrapper.ts";
import inspect from "object-inspect";

function StringifyParams(params: any[]): string {
  return params.reduce((acc: string, current: unknown) => {
    if (typeof current === "string") {
      return acc + " " + current;
    }

    if (typeof current === "object") {
      let stringObj = inspect(current, { indent: 2, quoteStyle: "double" });
      return acc + " " + stringObj;
    }

    return acc + " " + `${current}`;
  }, "");
}

const FILTERED = ["[Ant Design CSS-in-JS]", "locize.com"];

const originalConsole = {
  trace: console.trace.bind(console),
  debug: console.debug.bind(console),
  info: console.info.bind(console),
  warn: console.warn.bind(console),
  error: console.error.bind(console),
} as const;

function forwardConsole(
  fnName: "trace" | "debug" | "info" | "warn" | "error",
  logger: (message: string) => Promise<void>,
) {
  console[fnName] = (...params: any[]) => {
    originalConsole[fnName](...params);

    let message = StringifyParams(params);

    if (FILTERED.some((filter) => message.includes(filter))) {
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
    if (event.error instanceof Error) {
      console.error("Uncaught Error:", event.error);
    } else {
      console.error("Uncaught Error:", {
        message: event.message,
        file: event.filename,
        line: event.lineno,
        column: event.colno,
      });
    }
  },
  true,
);
