import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import Input from '../../components/ui/input.svelte';

describe('Input', () => {
	it('renders with placeholder', () => {
		render(Input, { placeholder: 'Search notes...' });
		expect(screen.getByPlaceholderText('Search notes...')).toBeInTheDocument();
	});

	it('applies the ui-input class', () => {
		render(Input, { placeholder: 'test' });
		const input = screen.getByPlaceholderText('test');
		expect(input.classList.contains('ui-input')).toBe(true);
	});

	it('merges custom class', () => {
		render(Input, { placeholder: 'test', class: 'my-custom' });
		const input = screen.getByPlaceholderText('test');
		expect(input.classList.contains('my-custom')).toBe(true);
		expect(input.classList.contains('ui-input')).toBe(true);
	});

	it('handles user typing', async () => {
		const user = userEvent.setup();
		render(Input, { placeholder: 'type here' });
		const input = screen.getByPlaceholderText('type here');
		await user.type(input, 'hello');
		expect(input).toHaveValue('hello');
	});

	it('passes disabled attribute', () => {
		render(Input, { placeholder: 'disabled', disabled: true });
		const input = screen.getByPlaceholderText('disabled');
		expect(input).toBeDisabled();
	});

	it('passes type attribute', () => {
		render(Input, { placeholder: 'email', type: 'email' });
		const input = screen.getByPlaceholderText('email');
		expect(input).toHaveAttribute('type', 'email');
	});
});
