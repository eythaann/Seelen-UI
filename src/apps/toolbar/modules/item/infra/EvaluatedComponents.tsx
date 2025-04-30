import { IconName } from '@icons';
import Sandbox from '@nyariv/sandboxjs';
import { FileIcon, Icon } from '@shared/components/Icon';
import { convertFileSrc } from '@tauri-apps/api/core';
import { memo, useEffect, useRef, useState } from 'react';
import { z } from 'zod';

import { EvaluateAction } from '../app';

interface SanboxedComponentProps {
  code: string;
  scope: Record<string, any>;
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
};

export const SanboxedComponent = memo(_SanboxedComponent);
function _SanboxedComponent({ code, scope }: SanboxedComponentProps) {
  const sandbox = useRef(new Sandbox());
  const [executor, setExecutor] = useState(() => sandbox.current.compile(code));

  useEffect(() => {
    sandbox.current = new Sandbox();
    const newExecutor = sandbox.current.compile(code);
    setExecutor(() => newExecutor);
  }, [code]);

  try {
    const content = executor({ ...scope, ...ComponentCreatorScope }).run();
    return <ElementsFromEvaluated content={content} />;
  } catch (error) {
    const { env: _, ...rest } = scope;
    console.error(error, { scope: rest });
    return <span>!?</span>;
  }
}

function ElementsFromEvaluated({ content }: { content: unknown }) {
  switch (typeof content) {
    case 'string':
      return <span>{content}</span>;
    case 'number':
    case 'boolean':
    case 'bigint':
      return <span>{String(content)}</span>;
    case 'object':
      if (content === null) {
        return null;
      }

      if (Array.isArray(content)) {
        return content.map((item: unknown, index: number) => {
          return <ElementsFromEvaluated key={index} content={item} />;
        });
      }

      if ('__type' in content) {
        if (content.__type === 'button') {
          return <EvaluatedButton {...EvaluatedButtonPropsSchema.parse(content)} />;
        }

        if (content.__type === 'react-icon') {
          return <EvaluatedReactIcon {...EvaluatedReactIconPropsSchema.parse(content)} />;
        }

        if (content.__type === 'image') {
          return <EvaluatedImage {...EvaluatedImagePropsSchema.parse(content)} />;
        }

        if (content.__type === 'app-icon') {
          return <EvaluatedAppIcon {...EvaluatedAppIconPropsSchema.parse(content)} />;
        }
      }

    default:
      return null;
  }
}

type EvaluatedButtonProps = z.infer<typeof EvaluatedButtonPropsSchema>;
const EvaluatedButtonPropsSchema = z.object({
  __type: z.literal('button').default('button'),
  content: z.unknown().nullish(),
  onClick: z.string().nullish(),
});

function EvaluatedButton({ content, onClick }: EvaluatedButtonProps) {
  return (
    <button
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
  __type: z.literal('react-icon').default('react-icon'),
  name: z.string(),
  size: z.number().optional(),
});

function EvaluatedReactIcon({ name, size }: EvaluatedReactIconProps) {
  return <Icon iconName={name as IconName} size={size} />;
}

type EvaluatedImageProps = z.infer<typeof EvaluatedImagePropsSchema>;
const EvaluatedImagePropsSchema = z.object({
  __type: z.literal('image').default('image'),
  url: z.string().nullish(),
  path: z.string().nullish(),
  size: z.union([z.string(), z.number()]).default('1rem'),
});

function EvaluatedImage({ url, path, size }: EvaluatedImageProps) {
  return (
    <img
      src={path ? convertFileSrc(path) : url || ''}
      style={{ width: size, height: size, objectFit: 'contain' }}
    />
  );
}

type EvaluatedAppIconProps = z.infer<typeof EvaluatedAppIconPropsSchema>;
const EvaluatedAppIconPropsSchema = z.object({
  __type: z.literal('app-icon').default('app-icon'),
  path: z.string().nullish(),
  umid: z.string().nullish(),
  size: z.union([z.string(), z.number()]).default('1rem'),
});

function EvaluatedAppIcon({ path, umid, size }: EvaluatedAppIconProps) {
  return <FileIcon path={path} umid={umid} style={{ width: size, height: size }} />;
}
