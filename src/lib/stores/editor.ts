import { writable } from 'svelte/store';
export const activeNotePath = writable<string>('');
