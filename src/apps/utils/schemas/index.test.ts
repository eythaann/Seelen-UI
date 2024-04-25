import { VariableConvention } from '.';

describe('VariableConvention', () => {
  test('snakeToCamel', () => {
    expect(VariableConvention.snakeToCamel('hello_world')).toBe('helloWorld');
    expect(VariableConvention.snakeToCamel('testing_snake_case_text')).toBe('testingSnakeCaseText');
    expect(VariableConvention.snakeToCamel('hello_world_1_2_3')).toBe('helloWorld123');
  });

  test('camelToSnake', () => {
    expect(VariableConvention.camelToSnake('HelloWorld')).toBe('hello_world');
    expect(VariableConvention.camelToSnake('helloWorld')).toBe('hello_world');
    expect(VariableConvention.camelToSnake('testingSnakeCaseText')).toBe('testing_snake_case_text');
    expect(VariableConvention.camelToSnake('helloWorld123')).toBe('hello_world123');
  });

  test('deepKeyParser', () => {
    const camelObject = [
      {
        helloWorld: {
          myWorld: 123,
        },
        testing: {
          camelCase: {
            deepCamelCase: 'text',
          },
          arrayCamelCase: [
            {
              deepCamelCase: 'text',
            },
            'text',
          ],
        },
      },
    ];

    const snakeObject = [
      {
        hello_world: {
          my_world: 123,
        },
        testing: {
          camel_case: {
            deep_camel_case: 'text',
          },
          array_camel_case: [
            {
              deep_camel_case: 'text',
            },
            'text',
          ],
        },
      },
    ];

    expect(VariableConvention.deepKeyParser(camelObject, VariableConvention.camelToSnake)).toEqual(snakeObject);
    expect(VariableConvention.deepKeyParser(snakeObject, VariableConvention.snakeToCamel)).toEqual(camelObject);
  });
});
