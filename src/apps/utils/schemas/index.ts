import z from 'zod';

export class VariableConvention {
  static snakeToCamel(text: string) {
    let camel = '';
    let prevCharIsDash = false;
    for (const char of text.split('')) {
      if (char === '_') {
        prevCharIsDash = true;
        continue;
      }
      if (prevCharIsDash) {
        camel += char.toUpperCase();
        prevCharIsDash = false;
      } else {
        camel += char;
      }
    }
    return camel;
  }

  static camelToSnake(text: string) {
    let snake = '';
    for (let i = 0; i < text.length; i++) {
      const char = text[i]!;
      if ((char === char.toLowerCase() && !char.match(/[0-9]/)) || i === 0) {
        snake += char.toLowerCase();
      } else {
        snake += `_${char.toLowerCase()}`;
      }
    }
    return snake;
  }

  static camelToUser(text: string) {
    return VariableConvention.camelToSnake(text).replace(/_/g, ' ');
  }

  static deepKeyParser(obj: anyObject, parser: (text: string) => string): anyObject {
    if (Array.isArray(obj)) {
      return obj.map((x) => {
        if (typeof x === 'object' && x != null) {
          return VariableConvention.deepKeyParser(x, parser);
        }
        return x;
      });
    }

    let newObj = {} as anyObject;
    for (const key in obj) {
      const value = obj[key];
      if (typeof value === 'object' && value != null) {
        newObj[parser(key)] = VariableConvention.deepKeyParser(value, parser);
      } else {
        newObj[parser(key)] = value;
      }
    }
    return newObj;
  }

  static fromSnakeToCamel(value: any): any {
    return VariableConvention.deepKeyParser(value, VariableConvention.snakeToCamel);
  }

  static fromCamelToSnake(value: any): any {
    return VariableConvention.deepKeyParser(value, VariableConvention.camelToSnake);
  }
}

export function parseAsCamel(schema: z.Schema, value: any) {
  return VariableConvention.fromSnakeToCamel(schema.parse(value));
}

export const CreatorInfoSchema = z.object({
  displayName: z.string().default('Unknown'),
  author: z.string().default('Unknown'),
  description: z.string().default('Empty'),
});
