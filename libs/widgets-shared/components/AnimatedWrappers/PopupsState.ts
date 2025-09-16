import { computed, signal } from "@preact/signals";

export const $open_popups = signal<Record<string, boolean>>({});
export const $there_are_open_popups = computed(() => Object.values($open_popups.value).some((v) => v));
