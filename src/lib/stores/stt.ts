import { writable } from 'svelte/store';

export const sttEnabled = writable<boolean>(false);
export const sttActive = writable<boolean>(false);
export const sttDevice = writable<string>('');
export const sttModelPath = writable<string>('');
