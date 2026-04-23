<script lang="ts">
import { Command } from 'bits-ui';
import { fileCreate, fileRead } from '$lib/api';
import { Dialog } from '$lib/components/ui';
import { appState, openFile, refreshFileTree } from '$lib/store.svelte';
import type { FileNode } from '$lib/types';

interface Props {
	onClose: () => void;
}

let { onClose }: Props = $props();
let open = $state(true);
let query = $state('');
let flatFiles: { name: string; path: string }[] = $state([]);

$effect(() => {
	if (!open) {
		onClose();
	}
});

$effect(() => {
	if (appState.system) {
		const tree = appState.fileTree;
		if (tree.length > 0) {
			flatFiles = flattenTree(tree);
		} else {
			refreshFileTree().then((t) => {
				flatFiles = flattenTree(t);
			});
		}
	}
});

function flattenTree(nodes: FileNode[]): { name: string; path: string }[] {
	const result: { name: string; path: string }[] = [];
	for (const node of nodes) {
		if (!node.is_directory) {
			result.push({ name: node.name, path: node.path });
		}
		if (node.children) {
			result.push(...flattenTree(node.children));
		}
	}
	return result;
}

function stripExt(name: string) {
	return name.replace(/\.md$/, '');
}

let filtered = $derived(
	query.trim() === ''
		? flatFiles
		: flatFiles.filter((f) =>
				f.name.toLowerCase().includes(query.toLowerCase()),
			),
);

let showCreate = $derived(
	query.trim() !== '' &&
		!filtered.some(
			(f) => stripExt(f.name).toLowerCase() === query.toLowerCase(),
		),
);

async function handleSelect(file?: { name: string; path: string }) {
	const target = file || filtered[0];
	if (!target && !showCreate) return;

	if (!target && showCreate) {
		if (!appState.system) return;
		const notesDir = `${appState.system.path}/notes`;
		const newPath = `${notesDir}/${query.replace(/\.md$/, '')}.md`;
		await fileCreate(newPath);
		const content = await fileRead(newPath);
		openFile(newPath, content);
		refreshFileTree();
	} else if (target) {
		const content = await fileRead(target.path);
		openFile(target.path, content);
	}
	open = false;
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape') {
		open = false;
	}
}
</script>

<Dialog bind:open position="top" contentClass="find-dialog" showClose={false}>
  <Command.Root shouldFilter={false} class="cmd-root">
    <Command.Input
      class="cmd-input"
      bind:value={query}
      placeholder="Find or create note..."
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
          <span class="result-name">{stripExt(file.name)}</span>
        </Command.Item>
      {/each}
      {#if showCreate}
        <Command.Item
          class="cmd-item"
          style="color: var(--muted-3); font-style: italic; border-top: 1px solid var(--border);"
          value={`__create__${query}`}
          onSelect={() => handleSelect()}
        >
          Create "{query}"
        </Command.Item>
      {/if}
      {#if filtered.length === 0 && !showCreate}
        <Command.Empty class="cmd-empty">No notes found</Command.Empty>
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
