import type Sandbox from "@nyariv/sandboxjs";
import { evalSanboxed } from "libs/ui/svelte/utils/sandbox";
import { z } from "zod";

export enum ObjectComponentKind {
  Icon = "Icon",
  AppIcon = "AppIcon",
  Image = "Image",
  Button = "Button",
  Group = "Group",
}

const EvaluatedReactIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Icon).default(ObjectComponentKind.Icon),
  name: z.string(),
});

const EvaluatedAppIconPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.AppIcon).default(ObjectComponentKind.AppIcon),
  path: z.string().nullish(),
  umid: z.string().nullish(),
});

const EvaluatedImagePropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Image).default(ObjectComponentKind.Image),
  url: z.string().nullish(),
  path: z.string().nullish(),
});

const EvaluatedButtonPropsSchema = z.object({
  "@component": z.literal(ObjectComponentKind.Button).default(ObjectComponentKind.Button),
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
  onClick: z.string().nullish(),
});

const EvaluatedGroupPropsSchema = z.object({
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

type ParsedComponent =
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

const ComponentCreatorScope = {
  icon: (arg1?: unknown, arg2?: unknown) => EvaluatedReactIconPropsSchema.parse({ name: arg1, size: arg2 }),
  Icon: (arg: unknown) => EvaluatedReactIconPropsSchema.parse(arg),
  AppIcon: (arg: unknown) => EvaluatedAppIconPropsSchema.parse(arg),
  Image: (arg: unknown) => EvaluatedImagePropsSchema.parse(arg),
  Button: (arg: unknown) => EvaluatedButtonPropsSchema.parse(arg),
  Group: (arg: unknown) => EvaluatedGroupPropsSchema.parse(arg),
};

export function evalComponentSandboxed(
  executor: ReturnType<Sandbox["compile"]> | null,
  scope: Record<string, any>,
): unknown {
  return evalSanboxed(executor, { ...scope, ...ComponentCreatorScope });
}
