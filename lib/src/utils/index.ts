export function getRootElement() {
  const element = document.getElementById('root');
  if (!element) {
    throw new Error('Root element not found');
  }
  return element;
}

export class Rect {
  left = 0;
  top = 0;
  right = 0;
  bottom = 0;
}