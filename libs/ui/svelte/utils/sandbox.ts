import Sandbox from "@nyariv/sandboxjs";

/** Sandbox with `CanvasRenderingContext2D` whitelisted so sandboxed code can draw on a canvas. */
function createCanvasSandbox(): Sandbox {
  const prototypeWhitelist = new Map(Sandbox.SAFE_PROTOTYPES);
  prototypeWhitelist.set(CanvasRenderingContext2D, new Set());
  return new Sandbox({ prototypeWhitelist });
}

const SANDBOX = createCanvasSandbox();

export type SanboxedEval = ReturnType<Sandbox["compile"]>;
export function compileSandboxed(source?: string | null): SanboxedEval | null {
  if (!source) return null;
  try {
    return SANDBOX.compile(source, true);
  } catch (e) {
    console.error("Error compiling code:", e);
    return null;
  }
}

export function evalSanboxed(executor: SanboxedEval | null, scope: Record<string, any>): unknown {
  if (!executor) return null;
  try {
    return executor({ ...scope }).run();
  } catch (error) {
    console.error("Error executing sandboxed code:", error);
    return null;
  }
}

const canvasContexts = new WeakMap<HTMLCanvasElement, RenderingContext>();

/** Preserve the drawing-state reset expected by plugins without resizing a 2D canvas every draw. */
export function drawPluginCanvas(
  canvas: HTMLCanvasElement,
  cssWidth: number,
  cssHeight: number,
  executor: SanboxedEval,
  scope: Record<string, unknown>,
  isDarkMode: boolean,
): void {
  const width = Math.floor(cssWidth * window.devicePixelRatio);
  const height = Math.floor(cssHeight * window.devicePixelRatio);
  if (width < 1 || height < 1) return;

  const computed = getComputedStyle(canvas);
  const systemTokens = getSystemTokens(computed);
  const themeTokens = getThemeTokens(computed);
  const context = canvasContexts.get(canvas);
  const resized = canvas.width !== width || canvas.height !== height;
  if (canvas.width !== width) canvas.width = width;
  if (canvas.height !== height) canvas.height = height;
  if (!resized && context) {
    if (context instanceof CanvasRenderingContext2D && typeof context.reset === "function") {
      context.reset();
    } else {
      // Preserve the previous reset behavior for other context types.
      canvas.width = width;
    }
  }

  evalSanboxed(executor, {
    ...scope,
    isDarkMode,
    systemTokens,
    themeTokens,
    canvas: {
      getContext(contextId: string) {
        const context = canvas.getContext(contextId);
        if (context) canvasContexts.set(canvas, context);
        return context;
      },
      width,
      height,
    },
  });
}

export function evalToStr(content: unknown): string {
  switch (typeof content) {
    case "string":
      return content;
    case "number":
    case "boolean":
    case "bigint":
      return String(content);
    case "object":
      if (content === null) return "";
      if (Array.isArray(content)) return content.map(evalToStr).join("");
      return "";
    default:
      return "";
  }
}

export function getSystemTokens(computed: CSSStyleDeclaration) {
  return {
    accentLightestColor: computed.getPropertyValue("--system-accent-lightest-color"),
    accentLighterColor: computed.getPropertyValue("--system-accent-lighter-color"),
    accentLightColor: computed.getPropertyValue("--system-accent-light-color"),
    accentColor: computed.getPropertyValue("--system-accent-color"),
    accentDarkColor: computed.getPropertyValue("--system-accent-dark-color"),
    accentDarkerColor: computed.getPropertyValue("--system-accent-darker-color"),
    accentDarkestColor: computed.getPropertyValue("--system-accent-darkest-color"),
  };
}

export function getThemeTokens(computed: CSSStyleDeclaration) {
  return {
    foregroundColor: computed.getPropertyValue("--slu-std-fg-color"),
    foregroundSecondaryColor: computed.getPropertyValue("--slu-std-fg-secondary-color"),
    foregroundMutedColor: computed.getPropertyValue("--slu-std-fg-muted-color"),
    foregroundDisabledColor: computed.getPropertyValue("--slu-std-fg-disabled-color"),
    backgroundColor: computed.getPropertyValue("--slu-std-bg-color"),
  };
}
