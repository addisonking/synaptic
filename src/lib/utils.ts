/**
 * Join class names, filtering out falsy values.
 * Replaces tailwind-merge/clsx since we use custom CSS, not Tailwind.
 */
export function cn(...inputs: (string | undefined | null | false)[]) {
	return inputs.filter(Boolean).join(' ');
}
