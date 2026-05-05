<script lang="ts">
import { Command } from 'bits-ui';
import { fileCreate, fileRead, getTags } from '$lib/api';
import { Dialog } from '$lib/components/ui';
import { appState, openFile, refreshFileTree } from '$lib/store.svelte';
import type { FileNode, TagEntry } from '$lib/types';

interface Props {
	onClose: () => void;
}

let { onClose }: Props = $props();
let open = $state(true);
let query = $state('');
let flatFiles: { name: string; path: string }[] = $state([]);
let allTags: TagEntry[] = $state([]);

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
		getTags(appState.system.path)
			.then((tags) => {
				allTags = tags;
			})
			.catch(() => {});
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

const tagPrefix = $derived(
	query.startsWith('#') ? query.slice(1).toLowerCase() : null,
);

const filtered = $derived(
	(() => {
		if (tagPrefix !== null) {
			const matchingPaths = new Set<string>();
			for (const entry of allTags) {
				if (tagPrefix === '' || entry.tag.startsWith(tagPrefix)) {
					for (const note of entry.notes) {
						matchingPaths.add(note.path);
					}
				}
			}
			return flatFiles.filter((f) => matchingPaths.has(f.path));
		}
		if (query.trim() === '') return flatFiles;
		return flatFiles.filter((f) =>
			f.name.toLowerCase().includes(query.toLowerCase()),
		);
	})(),
);

const showCreate = $derived(
	tagPrefix === null &&
		query.trim() !== '' &&
		!filtered.some(
			(f) => stripExt(f.name).toLowerCase() === query.toLowerCase(),
		),
);

const activeTagNames = $derived(
	tagPrefix !== null
		? allTags
				.filter((e) => tagPrefix === '' || e.tag.startsWith(tagPrefix))
				.map((e) => e.tag)
		: [],
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
      placeholder={tagPrefix !== null ? 'Filter by tag…' : 'Find or create note…'}
      onkeydown={handleKeydown}
      autofocus
    />
    {#if tagPrefix !== null && activeTagNames.length > 0}
      <div class="tag-filter-bar">
        {#each activeTagNames as tag}
          <span class="tag-pill">#{tag}</span>
        {/each}
        <span class="tag-count">{filtered.length} notes</span>
      </div>
    {/if}
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
        <Command.Empty class="cmd-empty">
          {tagPrefix !== null ? `No notes tagged #${tagPrefix}` : 'No notes found'}
        </Command.Empty>
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

  .tag-filter-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
  }

  .tag-pill {
    font-size: 11px;
    padding: 2px 8px;
    background: #1a3a3a;
    color: #3a9a9a;
    border: 1px solid #2a5a5a;
    border-radius: 2px;
    font-family: var(--font);
  }

  .tag-count {
    font-size: 11px;
    color: var(--muted-3);
    margin-left: auto;
  }
</style>
