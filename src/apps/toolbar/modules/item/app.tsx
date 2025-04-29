import Sandbox from '@nyariv/sandboxjs';
import { SeelenCommand } from '@seelen-ui/lib';
import { invoke } from '@tauri-apps/api/core';
import { memo, useEffect, useRef } from 'react';

import { StringToElement } from './infra/StringElement';

interface SanboxedComponentProps {
  code: string;
  scope: Record<string, any>;
}

function _SanboxedComponent({ code, scope }: SanboxedComponentProps) {
  const sandbox = useRef(new Sandbox());
  const executor = useRef(sandbox.current.compile(code));

  useEffect(() => {
    sandbox.current = new Sandbox();
    executor.current = sandbox.current.compile(code);
  }, [code]);

  try {
    const content = executor.current({ ...scope }).run();
    return <ElementsFromEvaluated content={content} />;
  } catch (error) {
    console.error(error, { scope });
    return <span>!?</span>;
  }
}

export const SanboxedComponent = memo(_SanboxedComponent);

export function ElementsFromEvaluated({ content }: { content: any }) {
  switch (typeof content) {
    case 'string':
      return <StringToElement text={content} />;
    case 'number':
    case 'boolean':
    case 'bigint':
      return <StringToElement text={content.toString()} />;
    case 'object':
      if (Array.isArray(content)) {
        return content.map((item: any, index: number) => {
          return <ElementsFromEvaluated key={index} content={item} />;
        });
      }
    default:
      return null;
  }
}

const ActionsScope = {
  open(path: string) {
    invoke(SeelenCommand.OpenFile, { path });
  },
  run(program: string, args: string[], workingDir: string) {
    invoke(SeelenCommand.Run, { program, args, workingDir });
  },
  copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  },
};

export async function EvaluateAction(code: string, scope: Record<string, any>) {
  const sandbox = new Sandbox();
  const executor = sandbox.compileAsync(code);
  await executor({ ...scope, ...ActionsScope }).run();
}
