async function WaitForTauriInternals(): Promise<void> {
  await new Promise<void>((resolve) => {
    const checkInterval = setInterval(() => {
      if ('__TAURI_INTERNALS__' in window) {
        clearInterval(checkInterval);
        resolve();
      }
    }, 10);
  });
}

await WaitForTauriInternals();

const script = document.createElement('script');
script.src = './index.js';
script.type = 'module';
script.defer = true;
document.head.appendChild(script);

export {};
