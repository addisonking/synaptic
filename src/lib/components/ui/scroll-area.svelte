<script lang="ts">
  import { ScrollArea } from "bits-ui";
  import type { Snippet } from "svelte";
  import { cn } from "$lib/utils";

  interface Props {
    children: Snippet;
    class?: string;
    orientation?: "both" | "vertical" | "horizontal";
  }

  let { children, class: className, orientation = "vertical" }: Props = $props();
</script>

<ScrollArea.Root class={cn("scroll-root", className)}>
  <ScrollArea.Viewport class="scroll-viewport">
    {@render children()}
  </ScrollArea.Viewport>
  <ScrollArea.Scrollbar class="scroll-scrollbar" {orientation}>
    <ScrollArea.Thumb class="scroll-thumb" />
  </ScrollArea.Scrollbar>
  <ScrollArea.Corner />
</ScrollArea.Root>

<style>
  :global(.scroll-root) {
    position: relative;
    overflow: hidden;
  }
  :global(.scroll-viewport) {
    width: 100%;
    height: 100%;
    overflow: auto;
  }
  :global(.scroll-scrollbar) {
    display: flex;
    user-select: none;
    touch-action: none;
    padding: 2px;
    background: transparent;
    transition: background 160ms ease-out;
  }
  :global(.scroll-scrollbar[data-orientation="vertical"]) {
    width: 10px;
    flex-direction: column;
  }
  :global(.scroll-scrollbar[data-orientation="horizontal"]) {
    height: 10px;
    flex-direction: row;
  }
  :global(.scroll-thumb) {
    flex: 1;
    background: var(--surface-3);
    border-radius: 0;
    position: relative;
  }
  :global(.scroll-thumb:hover) {
    background: var(--muted-2);
  }
</style>
