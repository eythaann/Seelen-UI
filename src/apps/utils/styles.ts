type Args = undefined | string | { [x: string]: boolean | null | undefined };
export const cx = (...args: Args[]): string => {
  return args.map((arg) => {
    if (!arg) {
      return;
    }

    if (typeof arg === 'string') {
      return arg;
    }

    let classnames = '';
    Object.keys(arg).forEach((key) => {
      if (arg[key]) {
        classnames += ` ${key}`;
      }
    });

    return classnames.trimStart();
  }).join(' ');
};