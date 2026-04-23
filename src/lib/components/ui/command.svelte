<script lang="ts">
  import { Command } from "bits-ui";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  interface Props {
    value?: string;
    onValueChange?: (value: string) => void;
    label?: string;
    children: Snippet;
    class?: string;
    shouldFilter?: boolean;
  }

  let {
    value = $bindable(""),
    onValueChange,
    label = "Command menu",
    children,
    class: className,
    shouldFilter = true,
  }: Props = $props();
</script>

<Command.Root
  bind:value
  {onValueChange}
  {label}
  {shouldFilter}
  class={cn("cmd-root", className)}
>
  {@render children()}
</Command.Root>

<style>
  :global(.cmd-root) {
    display: flex;
    flex-direction: column;
    width: 100%;
    background: var(--surface-1);
    overflow: hidden;
  }
  :global(.cmd-input) {
    width: 100%;
    padding: 16px;
    font-size: 14px;
    border: none;
    border-bottom: 1px solid var(--surface-2);
    background: var(--surface-1);
    color: var(--text-bright);
    outline: none;
    font-family: var(--font);
  }
  :global(.cmd-input::placeholder) {
    color: var(--muted-2);
  }
  :global(.cmd-list) {
    max-height: 420px;
    overflow-y: auto;
    overflow-x: hidden;
  }
  :global(.cmd-item) {
    width: 100%;
    text-align: left;
    padding: 10px 16px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    transition: background 80ms;
    display: flex;
    flex-direction: column;
    gap: 4px;
    cursor: pointer;
    font-family: var(--font);
  }
  :global(.cmd-item:hover),
  :global(.cmd-item[data-selected="true"]) {
    background: var(--surface-2);
  }
  :global(.cmd-empty) {
    padding: 24px;
    text-align: center;
    color: var(--muted-2);
    font-size: 13px;
  }
  :global(.cmd-group-heading) {
    padding: 8px 16px;
    font-size: 10px;
    color: var(--muted-2);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
</style>
