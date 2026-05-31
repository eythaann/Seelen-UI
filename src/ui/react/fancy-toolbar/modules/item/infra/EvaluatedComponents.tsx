import Sandbox from "@nyariv/sandboxjs";
import { FileIcon, Icon } from "libs/ui/react/components/Icon/index.tsx";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useComputed } from "@preact/signals";
import { memo, useCallback, useMemo } from "preact/compat";
import { z } from "zod";

import { EvaluateAction } from "../app/actionEvaluator.ts";
import { $tray_icons } from "../../shared/state/systemTray.ts";

interface SanboxedComponentProps {
  code: string;
  scope: Record<string, any>;
}

enum ObjectComponentKind {
  Icon = "Icon",
  AppIcon = "AppIcon",
  Image = "Image",
  TrayIcon = "TrayIcon",
  Button = "Button",
  Group = "Group",
}

const ComponentCreatorScope = {
  icon: (arg1?: unknown, arg2?: unknown) => {
    return EvaluatedReactIconPropsSchema.parse({
      name: arg1,
      size: arg2,
    });
  },
  Icon: (arg: unknown) => EvaluatedReactIconPropsSchema.parse(arg),
  AppIcon: (arg: unknown) => EvaluatedAppIconPropsSchema.parse(arg),
  Image: (arg: unknown) => EvaluatedImagePropsSchema.parse(arg),
  TrayIcon: (arg: unknown) => EvaluatedTrayIconPropsSchema.parse(arg),
  Button: (arg: unknown) => EvaluatedButtonPropsSchema.parse(arg),
  Group: (arg: unknown) => EvaluatedGroupPropsSchema.parse(arg),
};

// Global cache for compiled code to avoid recompiling the same template
// across multiple items or component re-mounts
const compiledCodeCache = new Map<string, ReturnType<typeof compileCodeInternal>>();

function compileCodeInternal(code: string) {
  try {
    const sandbox = new Sandbox();
    return {
      sandbox,
      executor: sandbox.compile(code),
    };
  } catch (e) {
    console.error("Error compiling code: ", e);
    return null;
  }
}

function compileCode(code: string) {
  if (compiledCodeCache.has(code)) {
    return compiledCodeCache.get(code)!;
  }

  const result = compileCodeInternal(code);
  compiledCodeCache.set(code, result);
  return result;
}

export function useSandboxedCode({ code, scope }: SanboxedComponentProps): unknown {
  const compiled = useMemo(() => compileCode(code), [code]);

  const computed = useMemo(() => {
    if (!compiled) {
      return null;
    }

    try {
      return compiled?.executor({ ...scope, ...ComponentCreatorScope }).run();
    } catch (error) {
      const { env: _, ...rest } = scope;

      console.error("Error while executing JS sandboxed code:", {
        error,
        code,
        scope: rest,
      });
      return null;
    }
  }, [compiled, scope]);

  return computed;
}

export function StringFromEvaluated({ content }: { content: unknown }): string {
  switch (typeof content) {
    case "string":
      return content;
    case "number":
    case "boolean":
    case "bigint":
      return String(content);
    case "object":
      if (content === null) {
        return "";
      }

      if (Array.isArray(content)) {
        return content
          .map((item: unknown) => {
            return StringFromEvaluated({ content: item });
          })
          .join("");
      }

      return "";
    default:
      return "";
  }
}

// Memoized version to prevent re-renders when content hasn't changed
export const ElementsFromEvaluated = memo(function ElementsFromEvaluated({
  content,
}: {
  content: unknown;
}) {
  // Use useMemo for array mapping to generate stable keys
  const renderContent = useMemo(() => {
    switch (typeof content) {
      case "string":
        return <span>{content}</span>;
      case "number":
      case "boolean":
      case "bigint":
        return <span>{String(content)}</span>;
      case "object":
        if (content === null) {
          return null;
        }

        if (Array.isArray(content)) {
          return content.map((item: unknown, index: number) => {
            // Create a stable key based on content if possible
            const key = typeof item === "object" && item !== null && "@component" in item
              ? `${(item as any)["@component"]}-${index}`
              : index;
            return <ElementsFromEvaluated key={key} content={item} />;
          });
        }

        if ("@component" in content) {
          switch (content["@component"]) {
            case ObjectComponentKind.Icon:
              return <EvaluatedReactIcon {...EvaluatedReactIconPropsSchema.parse(content)} />;
            case ObjectComponentKind.AppIcon:
              return <EvaluatedAppIcon {...EvaluatedAppIconPropsSchema.parse(content)} />;
            case ObjectComponentKind.Image:
              return <EvaluatedImage {...EvaluatedImagePropsSchema.parse(content)} />;
            case ObjectComponentKind.TrayIcon:
              return <EvaluatedTrayIcon {...EvaluatedTrayIconPropsSchema.parse(content)} />;
            case ObjectComponentKind.Button:
              return <EvaluatedButton {...EvaluatedButtonPropsSchema.parse(content)} />;
            case ObjectComponentKind.Group:
              return <EvaluatedGroup {...EvaluatedGroupPropsSchema.parse(content)} />;
            default:
              return null;
          }
        }

        return null;
      default:
        return null;
    }
  }, [content]);

  return renderContent;
});

type EvaluatedButtonProps = z.infer<typeof EvaluatedButtonPropsSchema>;
const EvaluatedButtonPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Button).default(ObjectComponentKind.Button),
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
  onClick: z.string().nullish(),
});
const EvaluatedButton = memo(function EvaluatedButton({
  style,
  content,
  onClick,
}: EvaluatedButtonProps) {
  const handleClick = useCallback(() => {
    if (onClick) {
      EvaluateAction(onClick, {});
    }
  }, [onClick]);

  return (
    <button data-skin="transparent" style={style} onClick={handleClick}>
      <ElementsFromEvaluated content={content} />
    </button>
  );
});

type EvaluatedReactIconProps = z.infer<typeof EvaluatedReactIconPropsSchema>;
const EvaluatedReactIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Icon).default(ObjectComponentKind.Icon),
  name: z.string(),
});
const EvaluatedReactIcon = memo(function EvaluatedReactIcon({ name }: EvaluatedReactIconProps) {
  return <Icon iconName={name as any} />;
});

type EvaluatedImageProps = z.infer<typeof EvaluatedImagePropsSchema>;
const EvaluatedImagePropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Image).default(ObjectComponentKind.Image),
  url: z.string().nullish(),
  path: z.string().nullish(),
});
const EvaluatedImage = memo(function EvaluatedImage({ url, path }: EvaluatedImageProps) {
  const imageSrc = useMemo(() => (path ? convertFileSrc(path) : url || ""), [path, url]);
  return <img src={imageSrc} />;
});

type EvaluatedTrayIconProps = z.infer<typeof EvaluatedTrayIconPropsSchema>;
const EvaluatedTrayIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.TrayIcon).default(ObjectComponentKind.TrayIcon),
  // Serialized SysTrayIconId (JSON.stringify of the icon's stable id).
  id: z.string(),
});
const EvaluatedTrayIcon = memo(function EvaluatedTrayIcon({ id }: EvaluatedTrayIconProps) {
  // Resolve the icon live from the shared tray state so it updates without
  // rewriting the (persisted) toolbar item.
  const resolved = useComputed(() => {
    const item = $tray_icons.value.find((it) => JSON.stringify(it.stable_id) === id);
    if (!item?.icon_path) return null;
    // Cache-bust with the image hash so the icon refreshes when its content changes.
    return convertFileSrc(item.icon_path) + `?hash=${item.icon_image_hash || "null"}`;
  });

  const src = resolved.value;
  // No live tray icon (app closed / not running yet): render nothing so the
  // slot collapses (see CSS). The toolbar item itself stays in place, so the
  // icon reappears in the same spot once the app is running again.
  if (!src) {
    return null;
  }
  return <img src={src} />;
});

type EvaluatedAppIconProps = z.infer<typeof EvaluatedAppIconPropsSchema>;
const EvaluatedAppIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.AppIcon).default(ObjectComponentKind.AppIcon),
  path: z.string().nullish(),
  umid: z.string().nullish(),
});
const EvaluatedAppIcon = memo(function EvaluatedAppIcon({ path, umid }: EvaluatedAppIconProps) {
  return <FileIcon path={path} umid={umid} />;
});

type EvaluatedGroupProps = z.infer<typeof EvaluatedGroupPropsSchema>;
const EvaluatedGroupPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Group).default(ObjectComponentKind.Group),
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
});
const EvaluatedGroup = memo(function EvaluatedGroup({ content, style }: EvaluatedGroupProps) {
  return (
    <div style={style}>
      <ElementsFromEvaluated content={content} />
    </div>
  );
});
