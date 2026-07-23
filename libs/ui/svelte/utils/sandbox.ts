import Sandbox from "@nyariv/sandboxjs";

/** Sandbox with `CanvasRenderingContext2D` whitelisted so sandboxed code can draw on a canvas. */
export function createCanvasSandbox(): Sandbox {
  const prototypeWhitelist = new Map(Sandbox.SAFE_PROTOTYPES);
  prototypeWhitelist.set(CanvasRenderingContext2D, new Set());
  return new Sandbox({ prototypeWhitelist });
}

type SanboxedEval = ReturnType<Sandbox["compile"]>;
export function compileSandboxed(sandbox: Sandbox, source?: string | null): SanboxedEval | null {
  if (!source) return null;
  try {
    return sandbox.compile(source);
  } catch (e) {
    console.error("Error compiling code:", e);
    return null;
  }
}

export function evalSanboxed(
  executor: SanboxedEval | null,
  scope: Record<string, any>,
): unknown {
  if (!executor) return null;
  try {
    return executor({ ...scope }).run();
  } catch (error) {
    console.error("Error executing sandboxed code:", error);
    return null;
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
