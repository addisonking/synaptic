import { describe, expect, it } from 'vitest';
import { parseFrontmatter } from '../markdown';

describe('parseFrontmatter', () => {
	it('returns empty tags and full body when no frontmatter', () => {
		const result = parseFrontmatter('# Hello\n\nWorld');
		expect(result.tags).toEqual([]);
		expect(result.body).toBe('# Hello\n\nWorld');
	});

	it('parses inline array tags', () => {
		const result = parseFrontmatter('---\ntags: [foo, bar, baz]\n---\n# Body');
		expect(result.tags).toEqual(['foo', 'bar', 'baz']);
		expect(result.body).toBe('# Body');
	});

	it('parses comma-separated tags', () => {
		const result = parseFrontmatter('---\ntags: foo, bar, baz\n---\ncontent');
		expect(result.tags).toEqual(['foo', 'bar', 'baz']);
	});

	it('parses single tag', () => {
		const result = parseFrontmatter('---\ntag: hello-world\n---\nbody');
		expect(result.tags).toEqual(['hello-world']);
	});

	it('parses multi-line YAML array tags', () => {
		const frontmatter = [
			'---',
			'tags:',
			'  - alpha',
			'  - beta',
			'  - gamma',
			'---',
			'body text',
		].join('\n');
		const result = parseFrontmatter(frontmatter);
		expect(result.tags).toEqual(['alpha', 'beta', 'gamma']);
	});

	it('returns empty tags when frontmatter has no tags field', () => {
		const result = parseFrontmatter('---\ntitle: Just a title\n---\nbody');
		expect(result.tags).toEqual([]);
	});

	it('handles frontmatter with only dashes', () => {
		// ---\n---\nbody is ambiguous — the regex sees it as no frontmatter
		// because after the first ---\n, the remaining has no leading \n before ---
		const result = parseFrontmatter('---\n---\nbody');
		expect(result.tags).toEqual([]);
	});
});
