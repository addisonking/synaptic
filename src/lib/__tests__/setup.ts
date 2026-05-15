import { cleanup } from '@testing-library/svelte';
import '@testing-library/jest-dom/vitest';
import { afterEach, vi } from 'vitest';

afterEach(cleanup);

// mock browser APIs jsdom might not have
if (typeof HTMLDialogElement === 'undefined') {
	// biome-ignore lint/suspicious/noExplicitAny: polyfill for test env
	(globalThis as any).HTMLDialogElement = class {};
}

// localStorage mock
const store = new Map<string, string>();
Object.defineProperty(globalThis, 'localStorage', {
	value: {
		getItem: (key: string) => store.get(key) ?? null,
		setItem: (key: string, value: string) => store.set(key, value),
		removeItem: (key: string) => store.delete(key),
		clear: () => store.clear(),
		get length() {
			return store.size;
		},
		key: (index: number) => [...store.keys()][index] ?? null,
	},
	writable: true,
});

// mock tauri invoke — tests import from this instead of the real module
vi.mock('@tauri-apps/api/core', () => ({
	invoke: vi.fn(),
	Channel: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn(),
}));

vi.mock('@tauri-apps/api/webview', () => ({
	getCurrentWebview: vi.fn(() => ({
		setZoom: vi.fn(() => Promise.resolve()),
	})),
}));
