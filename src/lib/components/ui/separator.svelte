<script lang="ts">
import { Separator } from 'bits-ui';
import { cn } from '$lib/utils';

interface Props {
	class?: string;
	orientation?: 'horizontal' | 'vertical';
}

let { class: className, orientation = 'horizontal' }: Props = $props();
</script>

<Separator.Root
  class={cn(
    "ui-separator",
    orientation === "horizontal" ? "ui-separator-horizontal" : "ui-separator-vertical",
    className
  )}
  {orientation}
/>

<style>
  :global(.ui-separator) {
    flex-shrink: 0;
    background: var(--border);
  }
  :global(.ui-separator-horizontal) {
    height: 1px;
    width: 100%;
  }
  :global(.ui-separator-vertical) {
    width: 1px;
    height: 100%;
  }
</style>
