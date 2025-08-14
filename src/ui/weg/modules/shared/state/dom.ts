import { signal } from '@preact/signals';
import { getRootContainer } from '@shared/index';

export const $root_hovered = signal(false);
getRootContainer()?.addEventListener('mouseenter', () => ($root_hovered.value = true));
getRootContainer()?.addEventListener('mouseleave', () => ($root_hovered.value = false));
