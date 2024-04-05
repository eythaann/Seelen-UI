type Args = undefined | string | { [x: string]: boolean | null | undefined };
export const cx = (...args: Args[]): string => {
  return args.map((arg) => {
    if (!arg) {
      return;
    }

    if (typeof arg === 'string') {
      return arg;
    }

    return Object.keys(arg).map((key) => arg[key] ? key : '').join(' ');
  }).join(' ');
};