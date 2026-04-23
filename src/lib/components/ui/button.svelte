<script lang="ts">
import type { Button } from 'bits-ui';
import type { Snippet } from 'svelte';
import { cn } from '$lib/utils';

type Variant =
	| 'default'
	| 'primary'
	| 'secondary'
	| 'ghost'
	| 'destructive'
	| 'outline';
type Size = 'default' | 'sm' | 'lg' | 'icon';

interface Props extends Button.RootProps {
	variant?: Variant;
	size?: Size;
	children?: Snippet;
}

let {
	variant = 'default',
	size = 'default',
	class: className,
	children,
	...rest
}: Props = $props();

const variantClasses: Record<Variant, string> = {
	default: 'btn-default',
	primary: 'btn-primary',
	secondary: 'btn-secondary',
	ghost: 'btn-ghost',
	destructive: 'btn-destructive',
	outline: 'btn-outline',
};

const sizeClasses: Record<Size, string> = {
	default: 'btn-size-default',
	sm: 'btn-size-sm',
	lg: 'btn-size-lg',
	icon: 'btn-size-icon',
};
</script>

<Button.Root
  class={cn("btn", variantClasses[variant], sizeClasses[size], className)}
  {...rest}
>
  {@render children?.()}
</Button.Root>

<style>
  :global(.btn) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    font-family: var(--font);
    font-size: 12px;
    cursor: pointer;
    border: none;
    background: none;
    color: inherit;
    transition: background 80ms, border-color 80ms, opacity 80ms;
    white-space: nowrap;
  }

  :global(.btn:disabled) {
    opacity: 0.4;
    cursor: not-allowed;
  }

  :global(.btn-default) {
    background: var(--surface-2);
    color: var(--text-bright);
    border: 1px solid var(--surface-3);
  }
  :global(.btn-default:hover:not(:disabled)) {
    background: var(--surface-3);
  }

  :global(.btn-primary) {
    background: var(--surface-2);
    color: var(--text-bright);
    border: 1px solid var(--surface-3);
  }
  :global(.btn-primary:hover:not(:disabled)) {
    background: var(--surface-3);
    border-color: var(--muted-2);
  }

  :global(.btn-secondary) {
    background: transparent;
    color: var(--text);
    border: 1px solid var(--surface-2);
  }
  :global(.btn-secondary:hover:not(:disabled)) {
    background: var(--surface-1);
    border-color: var(--surface-3);
  }

  :global(.btn-ghost) {
    background: transparent;
    color: var(--text);
    border: 1px solid transparent;
  }
  :global(.btn-ghost:hover:not(:disabled)) {
    background: var(--surface-1);
    border-color: var(--surface-2);
  }

  :global(.btn-destructive) {
    background: rgba(204, 68, 68, 0.1);
    color: var(--error);
    border: 1px solid var(--error);
  }
  :global(.btn-destructive:hover:not(:disabled)) {
    background: rgba(204, 68, 68, 0.2);
  }

  :global(.btn-outline) {
    background: transparent;
    color: var(--text);
    border: 1px solid var(--surface-2);
  }
  :global(.btn-outline:hover:not(:disabled)) {
    background: var(--surface-1);
    border-color: var(--surface-3);
  }

  :global(.btn-size-default) {
    padding: 8px 16px;
    font-size: 12px;
  }
  :global(.btn-size-sm) {
    padding: 5px 12px;
    font-size: 11px;
  }
  :global(.btn-size-lg) {
    padding: 10px 20px;
    font-size: 13px;
  }
  :global(.btn-size-icon) {
    padding: 4px 8px;
    font-size: 13px;
    color: var(--muted-2);
    border: 1px solid transparent;
    background: transparent;
  }
  :global(.btn-size-icon:hover:not(:disabled)) {
    color: var(--text-bright);
    background: var(--surface-2);
  }
</style>
