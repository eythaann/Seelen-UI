import Sandbox from "@nyariv/sandboxjs";
import { FileIcon, Icon } from "@shared/components/Icon";
import { IconName } from "@shared/components/Icon/icons";
import { convertFileSrc } from "@tauri-apps/api/core";
import { memo, useEffect, useState } from "react";
import { z } from "zod";

import { EvaluateAction } from "../app";

interface SanboxedComponentProps {
  code: string;
  scope: Record<string, any>;
}

enum ObjectComponentKind {
  Icon = "Icon",
  AppIcon = "AppIcon",
  Image = "Image",
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
  Button: (arg: unknown) => EvaluatedButtonPropsSchema.parse(arg),
  Group: (arg: unknown) => EvaluatedGroupPropsSchema.parse(arg),
};

function compileCode(code: string) {
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

export const SanboxedComponent = memo(_SanboxedComponent);
function _SanboxedComponent({ code, scope }: SanboxedComponentProps) {
  const [compiled, setCompiled] = useState(() => compileCode(code));

  useEffect(() => {
    setCompiled(compileCode(code));
  }, [code]);

  if (!compiled) {
    return <span>!?</span>;
  }

  try {
    const content = compiled.executor({ ...scope, ...ComponentCreatorScope }).run();
    return <ElementsFromEvaluated content={content} />;
  } catch (_error) {
    const { env: _, ...rest } = scope;
    console.error("Error executing component:", { scope: rest });
    return <span>!?</span>;
  }
}

function ElementsFromEvaluated({ content }: { content: unknown }) {
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
          return <ElementsFromEvaluated key={index} content={item} />;
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
}

type EvaluatedButtonProps = z.infer<typeof EvaluatedButtonPropsSchema>;
const EvaluatedButtonPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Button).default(ObjectComponentKind.Button),
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
  onClick: z.string().nullish(),
});
function EvaluatedButton({ style, content, onClick }: EvaluatedButtonProps) {
  return (
    <button
      style={style}
      onClick={() => {
        if (onClick) {
          EvaluateAction(onClick, {});
        }
      }}
    >
      <ElementsFromEvaluated content={content} />
    </button>
  );
}

type EvaluatedReactIconProps = z.infer<typeof EvaluatedReactIconPropsSchema>;
const EvaluatedReactIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Icon).default(ObjectComponentKind.Icon),
  name: z.string(),
  size: z.number().optional(),
});
function EvaluatedReactIcon({ name, size }: EvaluatedReactIconProps) {
  return <Icon iconName={name as IconName} size={size} />;
}

type EvaluatedImageProps = z.infer<typeof EvaluatedImagePropsSchema>;
const EvaluatedImagePropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Image).default(ObjectComponentKind.Image),
  url: z.string().nullish(),
  path: z.string().nullish(),
  size: z.union([z.string(), z.number()]).default("1rem"),
});
function EvaluatedImage({ url, path, size }: EvaluatedImageProps) {
  return (
    <img
      src={path ? convertFileSrc(path) : url || ""}
      style={{ width: size, height: size, objectFit: "contain" }}
    />
  );
}

type EvaluatedAppIconProps = z.infer<typeof EvaluatedAppIconPropsSchema>;
const EvaluatedAppIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.AppIcon).default(ObjectComponentKind.AppIcon),
  path: z.string().nullish(),
  umid: z.string().nullish(),
  size: z.union([z.string(), z.number()]).default("1rem"),
});
function EvaluatedAppIcon({ path, umid, size }: EvaluatedAppIconProps) {
  return <FileIcon path={path} umid={umid} style={{ width: size, height: size }} />;
}

type EvaluatedGroupProps = z.infer<typeof EvaluatedGroupPropsSchema>;
const EvaluatedGroupPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Group).default(ObjectComponentKind.Group),
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
});
function EvaluatedGroup({ content, style }: EvaluatedGroupProps) {
  return (
    <div style={style}>
      <ElementsFromEvaluated content={content} />
    </div>
  );
}
