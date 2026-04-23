<script lang="ts">
import type { HTMLInputAttributes } from 'svelte/elements';
import { cn } from '$lib/utils';

interface Props extends HTMLInputAttributes {
	class?: string;
}

let { class: className, value = $bindable(), ...rest }: Props = $props();
</script>

<input
  class={cn("ui-input", className)}
  bind:value
  {...rest}
/>

<style>
  :global(.ui-input) {
    width: 100%;
    font-family: var(--font);
    background: var(--bg);
    color: var(--text);
    border: 1px solid var(--surface-2);
    padding: 8px 12px;
    font-size: 13px;
    outline: none;
    transition: border-color 80ms;
  }
  :global(.ui-input:focus) {
    border-color: var(--muted-3);
  }
  :global(.ui-input::placeholder) {
    color: var(--muted-2);
  }
  :global(.ui-input:disabled) {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
