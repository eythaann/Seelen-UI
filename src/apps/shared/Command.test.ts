import { parseCommand } from './Command';

describe('parseCommand', () => {
  it('should parse a simple command without arguments', () => {
    const command = 'node';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: 'node',
      args: [],
    });
  });

  it('should parse a command with simple arguments', () => {
    const command = 'node myScript.js --flag';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: 'node',
      args: ['myScript.js', '--flag'],
    });
  });

  it('should parse a command with arguments containing spaces in double quotes', () => {
    const command = 'node myScript.js "argument with spaces"';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: 'node',
      args: ['myScript.js', 'argument with spaces'],
    });
  });

  it('should parse a command with arguments containing spaces in single quotes', () => {
    const command = 'node myScript.js \'single quoted argument\'';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: 'node',
      args: ['myScript.js', 'single quoted argument'],
    });
  });

  it('should parse a command with mixed quoted and unquoted arguments', () => {
    const command = 'node myScript.js --flag "argument with spaces" \'another argument\'';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: 'node',
      args: ['myScript.js', '--flag', 'argument with spaces', 'another argument'],
    });
  });

  it('should parse a command with no program (empty string)', () => {
    const command = '';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: undefined,
      args: [],
    });
  });

  it('should handle excessive spaces between arguments', () => {
    const command = 'node    myScript.js     --flag   ';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: 'node',
      args: ['myScript.js', '--flag'],
    });
  });

  it('should parse a command with a program path containing spaces', () => {
    const command = '"C:\\Program Files (x86)\\Steam\\steam.exe" -silent -login "user" "password"';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: 'C:\\Program Files (x86)\\Steam\\steam.exe',
      args: ['-silent', '-login', 'user', 'password'],
    });
  });

  it('should parse a program path with spaces without quotes', () => {
    const command = 'C:\\Program Files (x86)\\Steam\\steam.exe';
    const result = parseCommand(command);
    expect(result).toEqual({
      program: 'C:\\Program Files (x86)\\Steam\\steam.exe',
      args: [],
    });
  });
});
