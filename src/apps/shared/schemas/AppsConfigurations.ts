import { z } from 'zod';

export enum ApplicationIdentifier {
  Exe = 'Exe',
  Class = 'Class',
  Title = 'Title',
  Path = 'Path',
}

export enum MatchingStrategy {
  Legacy = 'Legacy',
  Equals = 'Equals',
  StartsWith = 'StartsWith',
  EndsWith = 'EndsWith',
  Contains = 'Contains',
  Regex = 'Regex',
}

function stringInsensitiveToEnum<Enum extends Record<string, string>>(value: string, enumObj: Enum) {
  return Object.values(enumObj).find((v) => v.toLocaleLowerCase() === value.toLocaleLowerCase()) as Enum[keyof Enum] | undefined;
}

interface _IdWithIdentifier {
  id: string;
  kind: ApplicationIdentifier;
  matching_strategy: MatchingStrategy;
  negation: boolean;
  and: _IdWithIdentifier[];
  or: _IdWithIdentifier[];
}

export const IdWithIdentifierSchema = z.object({
  id: z.string().default('new-app.exe'),
  kind: z
    .string()
    .transform((arg) => stringInsensitiveToEnum(arg, ApplicationIdentifier))
    .default(ApplicationIdentifier.Exe),
  matching_strategy: z
    .string()
    .transform((arg) => stringInsensitiveToEnum(arg, MatchingStrategy))
    .default(MatchingStrategy.Equals),
  negation: z.boolean().default(false),
  and: z.array(z.lazy(() => IdWithIdentifierSchema)).default([]),
  or: z.array(z.lazy(() => IdWithIdentifierSchema)).default([]),
}) as z.ZodType<_IdWithIdentifier>;

export interface IdWithIdentifier {
  id: _IdWithIdentifier['id'];
  kind: _IdWithIdentifier['kind'];
  matchingStrategy: _IdWithIdentifier['matching_strategy'];
  negation: _IdWithIdentifier['negation'];
  and: IdWithIdentifier[];
  or: IdWithIdentifier[];
}

export class IdWithIdentifier {
  static default(): IdWithIdentifier {
    return {
      id: 'new-app.exe',
      kind: ApplicationIdentifier.Exe,
      matchingStrategy: MatchingStrategy.Equals,
      negation: false,
      and: [],
      or: [],
    };
  }
}