<script lang="ts">
import { Command } from 'bits-ui';
import { fileRead, scratchList } from '$lib/api';
import { Dialog } from '$lib/components/ui';
import { appState, openFile } from '$lib/store.svelte';
import type { ScratchEntry } from '$lib/types';

interface Props {
	onClose: () => void;
}

let { onClose }: Props = $props();
let open = $state(true);
let query = $state('');
let files: ScratchEntry[] = $state([]);

$effect(() => {
	if (!open) {
		onClose();
	}
});

$effect(() => {
	if (appState.system) {
		scratchList(appState.system.path).then((list) => {
			files = list;
		});
	}
});

function formatName(entry: ScratchEntry) {
	const isTimestamp = /^\d{4}-\d{2}-\d{2}-\d{2}-\d{2}-\d{2}$/.test(entry.name);
	if (isTimestamp) {
		const parts = entry.name.split('-');
		return `${parts[0]}-${parts[1]}-${parts[2]} ${parts[3]}:${parts[4]}:${parts[5]}`;
	}
	return entry.name.replace(/-/g, ' ');
}

let filtered = $derived(
	query.trim() === ''
		? files
		: files.filter((f) =>
				f.name.toLowerCase().replace(/-/g, ' ').includes(query.toLowerCase()),
			),
);

async function handleSelect(file?: ScratchEntry) {
	const target = file || filtered[0];
	if (!target) return;
	const content = await fileRead(target.path);
	openFile(target.path, content);
	open = false;
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape') {
		e.preventDefault();
		open = false;
	}
}
</script>

<Dialog bind:open position="top" contentClass="find-dialog" showClose={false}>
  <Command.Root shouldFilter={false} class="cmd-root">
    <Command.Input
      class="cmd-input"
      bind:value={query}
      placeholder="Search scratch notes..."
      onkeydown={handleKeydown}
      autofocus
    />
    <Command.List class="cmd-list">
      {#each filtered as file}
        <Command.Item
          class="cmd-item"
          value={file.path}
          onSelect={() => handleSelect(file)}
        >
          <span class="result-name">{formatName(file)}</span>
        </Command.Item>
      {/each}
      {#if filtered.length === 0}
        <Command.Empty class="cmd-empty">No scratch notes found</Command.Empty>
      {/if}
    </Command.List>
  </Command.Root>
</Dialog>

<style>
:global(.find-dialog) {
  width: 560px;
}
.result-name {
  color: var(--text-bright);
}
</style>