export async function registerDocumentEvents() {
  document.addEventListener('focusin', (event) => {
    const element = event.target as HTMLElement;
    if (element) {
      element.scrollIntoView({
        behavior: 'smooth',
        block: 'center',
        inline: 'nearest',
      });
    }
  });
}
