export function fromPackageRoot(...segments: string[]): string;
export function runPwshScript(name: string, args?: string): Promise<void>;
export function execPrinter(error, stdout, stderr): void;