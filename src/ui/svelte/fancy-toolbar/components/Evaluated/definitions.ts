import { evalSanboxed, type SanboxedEval } from "libs/ui/svelte/utils/sandbox";
import { z } from "zod";

const COMPONENT_KEY = "@component";

export enum ObjectComponentKind {
  Plain = "Plain",
  PlainList = "PlainList",

  Icon = "Icon",
  AppIcon = "AppIcon",
  Image = "Image",
  Button = "Button",
  Group = "Group",
}

export type EvaluatedReactIconProps = z.infer<typeof EvaluatedReactIconPropsSchema>;
const EvaluatedReactIconPropsSchema = z.object({
  name: z.string(),
});

export type EvaluatedAppIconProps = z.infer<typeof EvaluatedAppIconPropsSchema>;
const EvaluatedAppIconPropsSchema = z.object({
  path: z.string().nullish(),
  umid: z.string().nullish(),
});

export type EvaluatedImageProps = z.infer<typeof EvaluatedImagePropsSchema>;
const EvaluatedImagePropsSchema = z.object({
  url: z.string().nullish(),
  path: z.string().nullish(),
  /** change it to force the image to be reloaded (e.g. when the file is overwritten in place) */
  version: z.union([z.string(), z.number()]).nullish(),
});

export type EvaluatedButtonProps = z.infer<typeof EvaluatedButtonPropsSchema>;
const EvaluatedButtonPropsSchema = z.object({
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
  tooltip: z.string().nullish(),
  onClick: z.string().nullish(),
  onAuxClick: z.string().nullish(),
  onContextMenu: z.string().nullish(),
});

export type EvaluatedGroupProps = z.infer<typeof EvaluatedGroupPropsSchema>;
const EvaluatedGroupPropsSchema = z.object({
  style: z.record(z.any()).default({}),
  content: z.unknown().nullish(),
});

type ParsedComponent =
  | { kind: ObjectComponentKind.Plain; value: string }
  | { kind: ObjectComponentKind.PlainList; value: unknown[] }
  | { kind: ObjectComponentKind.Icon; props: EvaluatedReactIconProps }
  | { kind: ObjectComponentKind.Icon; props: EvaluatedReactIconProps }
  | { kind: ObjectComponentKind.AppIcon; props: EvaluatedAppIconProps }
  | { kind: ObjectComponentKind.Image; props: EvaluatedImageProps }
  | { kind: ObjectComponentKind.Button; props: EvaluatedButtonProps }
  | { kind: ObjectComponentKind.Group; props: EvaluatedGroupProps };

export function parseComponent(value: unknown): ParsedComponent | null {
  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean" ||
    typeof value === "bigint"
  ) {
    return {
      kind: ObjectComponentKind.Plain,
      value: String(value),
    };
  }

  if (typeof value !== "object" || value === null) return null;

  if (Array.isArray(value)) {
    return {
      kind: ObjectComponentKind.PlainList,
      value: value,
    };
  }

  if (!(COMPONENT_KEY in value)) return null;
  const obj = value as { [COMPONENT_KEY]: unknown; props?: unknown };

  switch (obj[COMPONENT_KEY]) {
    case ObjectComponentKind.Icon:
      return {
        kind: ObjectComponentKind.Icon,
        props: EvaluatedReactIconPropsSchema.parse(obj.props),
      };
    case ObjectComponentKind.AppIcon:
      return {
        kind: ObjectComponentKind.AppIcon,
        props: EvaluatedAppIconPropsSchema.parse(obj.props),
      };
    case ObjectComponentKind.Image:
      return { kind: ObjectComponentKind.Image, props: EvaluatedImagePropsSchema.parse(obj.props) };
    case ObjectComponentKind.Button:
      return {
        kind: ObjectComponentKind.Button,
        props: EvaluatedButtonPropsSchema.parse(obj.props),
      };
    case ObjectComponentKind.Group:
      return { kind: ObjectComponentKind.Group, props: EvaluatedGroupPropsSchema.parse(obj.props) };
    default:
      return null;
  }
}

const ComponentCreatorScope = {
  icon: (arg1?: unknown, arg2?: unknown) => ({
    [COMPONENT_KEY]: ObjectComponentKind.Icon,
    props: EvaluatedReactIconPropsSchema.parse({ name: arg1, size: arg2 }),
  }),
  Icon: (arg: unknown) => ({
    [COMPONENT_KEY]: ObjectComponentKind.Icon,
    props: EvaluatedReactIconPropsSchema.parse(arg),
  }),
  AppIcon: (arg: unknown) => ({
    [COMPONENT_KEY]: ObjectComponentKind.AppIcon,
    props: EvaluatedAppIconPropsSchema.parse(arg),
  }),
  Image: (arg: unknown) => ({
    [COMPONENT_KEY]: ObjectComponentKind.Image,
    props: EvaluatedImagePropsSchema.parse(arg),
  }),
  Button: (arg: unknown) => ({
    [COMPONENT_KEY]: ObjectComponentKind.Button,
    props: EvaluatedButtonPropsSchema.parse(arg),
  }),
  Group: (arg: unknown) => ({
    [COMPONENT_KEY]: ObjectComponentKind.Group,
    props: EvaluatedGroupPropsSchema.parse(arg),
  }),
};

export function evalComponentSandboxed(
  executor: SanboxedEval | null,
  scope: Record<string, any>,
): unknown {
  return evalSanboxed(executor, { ...scope, ...ComponentCreatorScope });
}
