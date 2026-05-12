<script lang="ts">
import { Dialog } from 'bits-ui';
import { X } from 'lucide-svelte';
import type { Snippet } from 'svelte';
import { cn } from '$lib/utils';

interface Props {
	open?: boolean;
	onOpenChange?: (open: boolean) => void;
	title?: string;
	description?: string;
	children?: Snippet;
	class?: string;
	contentClass?: string;
	showClose?: boolean;
	position?: 'center' | 'top';
	preventScroll?: boolean;
}

let {
	open = $bindable(false),
	onOpenChange,
	title,
	description,
	children,
	class: className,
	contentClass,
	showClose = true,
	position = 'center',
	preventScroll = true,
}: Props = $props();

function handleOpenChange(newOpen: boolean) {
	open = newOpen;
	onOpenChange?.(newOpen);
}
</script>

<Dialog.Root bind:open={open} onOpenChange={handleOpenChange}>
  <Dialog.Portal>
    <Dialog.Overlay class={cn("dialog-overlay", position === "top" && "dialog-overlay-top")} />
    <Dialog.Content
      class={cn(
        "dialog-content",
        position === "top" ? "dialog-content-top" : "dialog-content-center",
        contentClass
      )}
      {preventScroll}
    >
      {#if title}
        <Dialog.Title class="dialog-title">{title}</Dialog.Title>
      {/if}
      {#if description}
        <Dialog.Description class="dialog-description">{description}</Dialog.Description>
      {/if}
      <div class={cn("dialog-body", className)}>
        {@render children?.()}
      </div>
      {#if showClose}
        <Dialog.Close class="dialog-close">
          <X size={16} />
          <span class="sr-only">Close</span>
        </Dialog.Close>
      {/if}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<style>
  :global(.dialog-overlay) {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.85);
    z-index: 100;
  }
  :global(.dialog-overlay-top) {
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 120px;
  }

  :global(.dialog-content) {
    position: fixed;
    background: var(--surface-1);
    border: 1px solid var(--surface-2);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    z-index: 101;
    outline: none;
  }
  :global(.dialog-content-center) {
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    max-height: 80vh;
    display: flex;
    flex-direction: column;
  }
  :global(.dialog-content-top) {
    top: 120px;
    left: 50%;
    transform: translateX(-50%);
  }

  :global(.dialog-title) {
    font-size: 14px;
    color: var(--text-active);
    font-weight: 400;
    margin: 0;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }

  :global(.dialog-description) {
    font-size: 12px;
    color: var(--muted-2);
    padding: 0 20px 12px;
    margin-top: -8px;
  }

  :global(.dialog-body) {
    padding: 0;
    overflow-x: hidden;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  :global(.dialog-close) {
    position: absolute;
    top: 12px;
    right: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    color: var(--muted-2);
    background: transparent;
    border: none;
    cursor: pointer;
    transition: color 80ms;
  }
  :global(.dialog-close:hover) {
    color: var(--text-bright);
  }

  :global(.sr-only) {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
  }
</style>
