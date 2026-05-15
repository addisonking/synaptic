import path from 'node:path';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

export default defineConfig({
	plugins: [svelte()],
	test: {
		environment: 'jsdom',
		include: ['src/**/*.test.ts', 'src/**/*.test.svelte.ts'],
		setupFiles: ['src/lib/__tests__/setup.ts'],
		css: false,
	},
	resolve: {
		conditions: ['browser'],
		alias: {
			$lib: path.resolve(__dirname, 'src/lib'),
		},
	},
});
