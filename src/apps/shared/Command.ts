export function parseCommand(command: string) {
  const regex = /"(.*?)"|'(.*?)'|(\S+)/g;
  const matches = [];
  let match;

  // Extrae todas las coincidencias
  while ((match = regex.exec(command)) !== null) {
    matches.push(match[1] || match[2] || match[3]);
  }

  // Special handling for unquoted paths with spaces
  if (matches.length > 1 && /^[a-zA-Z]:\\/.test(matches[0]!)) {
    // Combine consecutive parts of the program path until the first argument
    let i = 1;
    while (i < matches.length && !matches[i]!.startsWith('-') && !matches[i]!.includes('=')) {
      matches[0] += ` ${matches[i]}`;
      matches.splice(i, 1);
    }
  }
  const program = matches.shift();
  const args = matches;

  return { program, args };
}