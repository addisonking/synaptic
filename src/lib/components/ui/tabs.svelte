<script lang="ts">
  import { Tabs } from "bits-ui";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  interface Props {
    value?: string;
    onValueChange?: (value: string) => void;
    items: { value: string; label: string }[];
    children?: Snippet;
    class?: string;
  }

  let {
    value = $bindable(),
    onValueChange,
    items,
    children,
    class: className,
  }: Props = $props();
</script>

<Tabs.Root bind:value {onValueChange} class={cn("tabs-root", className)}>
  <Tabs.List class="tabs-list">
    {#each items as item}
      <Tabs.Trigger value={item.value} class="tabs-trigger">
        {item.label}
      </Tabs.Trigger>
    {/each}
  </Tabs.List>
  {@render children?.()}
</Tabs.Root>

<style>
  :global(.tabs-root) {
    display: flex;
    flex-direction: column;
  }
  :global(.tabs-list) {
    display: flex;
    border-bottom: 1px solid var(--border);
    padding: 0 20px;
  }
  :global(.tabs-trigger) {
    padding: 10px 14px;
    font-size: 12px;
    color: var(--muted-2);
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: color 80ms, border-color 80ms;
    background: transparent;
    border-top: none;
    border-left: none;
    border-right: none;
    cursor: pointer;
    font-family: var(--font);
  }
  :global(.tabs-trigger:hover) {
    color: var(--text);
  }
  :global(.tabs-trigger[data-state="active"]) {
    color: var(--text-bright);
    border-color: var(--text-bright);
  }
</style>
