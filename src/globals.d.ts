declare module '*.module.css' {
  const classnames: Record<string, string>;
  export default classnames;
}

declare module '*.yml' {
  export default string;
}

interface ObjectConstructor {
  keys<T>(o: T): (keyof T)[];
}
