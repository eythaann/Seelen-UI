export function getRootElement() {
  const element = document.getElementById('root');
  if (!element) {
    throw new Error('Root element not found');
  }
  return element;
}
