import type Sandbox from "@nyariv/sandboxjs";
import { z } from "zod";

export enum ObjectComponentKind {
  Icon = "Icon",
  AppIcon = "AppIcon",
  Image = "Image",
  Button = "Button",
  Group = "Group",
}

export const ComponentCreatorScope = {
  icon: (arg1?: unknown, arg2?: unknown) => EvaluatedReactIconPropsSchema.parse({ name: arg1, size: arg2 }),
  Icon: (arg: unknown) => EvaluatedReactIconPropsSchema.parse(arg),
  AppIcon: (arg: unknown) => EvaluatedAppIconPropsSchema.parse(arg),
  Image: (arg: unknown) => EvaluatedImagePropsSchema.parse(arg),
  Button: (arg: unknown) => EvaluatedButtonPropsSchema.parse(arg),
  Group: (arg: unknown) => EvaluatedGroupPropsSchema.parse(arg),
};

export const EvaluatedReactIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Icon).default(ObjectComponentKind.Icon),
  name: z.string(),
});

export const EvaluatedAppIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.AppIcon).default(ObjectComponentKind.AppIcon),
  path: z.string().nullish(),
  umid: z.string().nullish(),
});

export const EvaluatedImagePropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Image).default(ObjectComponentKind.Image),
  url: z.string().nullish(),
  path: z.string().nullish(),
});

export const EvaluatedButtonPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Button).default(ObjectComponentKind.Button),
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
  onClick: z.string().nullish(),
});

export const EvaluatedGroupPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Group).default(ObjectComponentKind.Group),
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
});

export function stringFromEvaluated(content: unknown): string {
  switch (typeof content) {
    case "string":
      return content;
    case "number":
    case "boolean":
    case "bigint":
      return String(content);
    case "object":
      if (content === null) return "";
      if (Array.isArray(content)) return content.map(stringFromEvaluated).join("");
      return "";
    default:
      return "";
  }
}

export type ParsedComponent =
  | { kind: ObjectComponentKind.Icon; props: z.infer<typeof EvaluatedReactIconPropsSchema> }
  | { kind: ObjectComponentKind.AppIcon; props: z.infer<typeof EvaluatedAppIconPropsSchema> }
  | { kind: ObjectComponentKind.Image; props: z.infer<typeof EvaluatedImagePropsSchema> }
  | { kind: ObjectComponentKind.Button; props: z.infer<typeof EvaluatedButtonPropsSchema> }
  | { kind: ObjectComponentKind.Group; props: z.infer<typeof EvaluatedGroupPropsSchema> };

export function parseComponentProps(content: object): ParsedComponent | null {
  if (!("@component" in content)) return null;
  switch (content["@component"]) {
    case ObjectComponentKind.Icon:
      return {
        kind: ObjectComponentKind.Icon,
        props: EvaluatedReactIconPropsSchema.parse(content),
      };
    case ObjectComponentKind.AppIcon:
      return {
        kind: ObjectComponentKind.AppIcon,
        props: EvaluatedAppIconPropsSchema.parse(content),
      };
    case ObjectComponentKind.Image:
      return { kind: ObjectComponentKind.Image, props: EvaluatedImagePropsSchema.parse(content) };
    case ObjectComponentKind.Button:
      return {
        kind: ObjectComponentKind.Button,
        props: EvaluatedButtonPropsSchema.parse(content),
      };
    case ObjectComponentKind.Group:
      return { kind: ObjectComponentKind.Group, props: EvaluatedGroupPropsSchema.parse(content) };
    default:
      return null;
  }
}

export function compileSandboxed(sandbox: Sandbox, source?: string | null) {
  if (!source) return null;
  try {
    return sandbox.compile(source);
  } catch (e) {
    console.error("Error compiling code:", e);
    return null;
  }
}

export function evalComponentSandboxed(
  source: string,
  executor: ReturnType<Sandbox["compile"]> | null,
  scope: Record<string, any>,
): unknown {
  if (!executor) return null;
  try {
    return executor({ ...scope, ...ComponentCreatorScope }).run();
  } catch (error) {
    console.error("Error executing sandboxed code:", { error, source });
    return null;
  }
}
