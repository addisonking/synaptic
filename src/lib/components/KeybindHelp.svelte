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

const groups = [
	{
		section: 'Navigation',
		items: [
			{ key: '⌘P', action: 'Find or create note' },
			{ key: '⌘⇧P', action: 'Semantic search' },
			{ key: '⌘[', action: 'Go back' },
			{ key: '⌘]', action: 'Go forward' },
		],
	},
	{
		section: 'Notes',
		items: [
			{ key: '⌘N', action: 'New note' },
			{ key: '⌘⇧N', action: 'New scratch note' },
			{ key: '⌘K', action: 'Generate note with AI' },
			{ key: '⌘R', action: 'Rename current note' },
			{ key: '⌘⇧S', action: 'Search scratch notes' },
			{ key: 'Del ×2', action: 'Trash current note' },
		],
	},
	{
		section: 'Views',
		items: [
			{ key: '⌘G', action: 'Toggle graph view' },
			{ key: '⌘,', action: 'Settings' },
			{ key: '?', action: 'Keyboard shortcuts' },
			{ key: 'Esc', action: 'Close overlay' },
		],
	},
	{
		section: 'Zoom',
		items: [
			{ key: '⌘= / ⌘-', action: 'Zoom in / out' },
			{ key: '⌘0', action: 'Reset zoom' },
		],
	},
];
</script>

<Dialog bind:open title="Keyboard Shortcuts" contentClass="help-dialog">
  <div class="content">
    <table>
      <tbody>
        {#each groups as group}
          <tr class="section-header">
            <td colspan="2">{group.section}</td>
          </tr>
          {#each group.items as shortcut}
            <tr>
              <td class="key"><kbd>{shortcut.key}</kbd></td>
              <td class="action">{shortcut.action}</td>
            </tr>
          {/each}
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
  tr.section-header td {
    padding: 14px 0 4px;
    font-size: 10px;
    font-variant: small-caps;
    letter-spacing: 0.08em;
    color: var(--text-muted);
    border-bottom: none;
  }
  tr.section-header:first-child td {
    padding-top: 4px;
  }
  td {
    padding: 7px 0;
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
