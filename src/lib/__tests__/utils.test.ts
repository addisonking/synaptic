import { describe, expect, it } from 'vitest';
import { cn } from '../utils';

describe('cn', () => {
	it('joins class names', () => {
		expect(cn('a', 'b', 'c')).toBe('a b c');
	});

	it('filters falsy values', () => {
		expect(cn('a', undefined, null, false, 'b')).toBe('a b');
	});

	it('returns empty string for all falsy', () => {
		expect(cn(undefined, null, false)).toBe('');
	});

	it('handles single arg', () => {
		expect(cn('foo')).toBe('foo');
	});

	it('handles empty string args', () => {
		expect(cn('a', '', 'b')).toBe('a b');
	});
});
