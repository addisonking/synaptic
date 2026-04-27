<script lang="ts">
import { Button, Dialog } from '$lib/components/ui';

interface Props {
	onClose: () => void;
}

let { onClose }: Props = $props();
let open = $state(true);

$effect(() => {
	if (!open) {
		onClose();
	}
});

const shortcuts = [
	{ key: '⌘P', action: 'Find or create note' },
	{ key: '⌘⇧P', action: 'Semantic search' },
	{ key: '⌘N', action: 'New note' },
	{ key: '⌘⇧N', action: 'New scratch note' },
	{ key: '⌘⇧S', action: 'Search scratch notes' },
	{ key: '⌘G', action: 'Toggle graph view' },
	{ key: '⌘⇧G', action: 'Toggle ghost links' },
	{ key: '⌘[', action: 'Go back in history' },
	{ key: '⌘]', action: 'Go forward in history' },
	{ key: '⌘= / ⌘-', action: 'Zoom in / out' },
	{ key: '⌘0', action: 'Reset zoom' },
	{ key: '⌘,', action: 'Open settings' },
	{ key: '?', action: 'Show this help' },
	{ key: 'Esc', action: 'Close any overlay' },
	{ key: 'Delete (x2)', action: 'Trash current note' },
];
</script>

<Dialog bind:open title="Keyboard Shortcuts" contentClass="help-dialog">
  <div class="content">
    <table>
      <tbody>
        {#each shortcuts as shortcut}
          <tr>
            <td class="key"><kbd>{shortcut.key}</kbd></td>
            <td class="action">{shortcut.action}</td>
          </tr>
        {/each}
      </tbody>
    </table>
    <Button variant="default" onclick={() => (open = false)} class="close-btn">Close</Button>
  </div>
</Dialog>

<style>
  .content {
    padding: 0 24px 24px;
    width: 420px;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 20px;
  }
  tr {
    border-bottom: 1px solid var(--border);
  }
  tr:last-child {
    border-bottom: none;
  }
  td {
    padding: 8px 0;
    font-size: 13px;
  }
  .key {
    white-space: nowrap;
    padding-right: 16px;
    width: 1px;
  }
  .key kbd {
    background: var(--surface-2);
    border: 1px solid var(--surface-3);
    padding: 2px 8px;
    font-family: var(--font);
    font-size: 11px;
    color: var(--text-bright);
  }
  .action {
    color: var(--text);
  }
  :global(.close-btn) {
    width: 100%;
  }
</style>
