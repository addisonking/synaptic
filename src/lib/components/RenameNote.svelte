<script lang="ts">
import { fileRead, renameNote } from '$lib/api';
import { Button, Dialog, Input } from '$lib/components/ui';
import { appState, openFile, refreshFileTree } from '$lib/store.svelte';

interface Props {
	onClose: () => void;
}

let { onClose }: Props = $props();
let open = $state(true);
let name = $state('');
let error = $state('');
let loading = $state(false);

$effect(() => {
	if (!open) onClose();
});

$effect(() => {
	const stem =
		appState.openFilePath?.split('/').pop()?.replace(/\.md$/, '') ?? '';
	name = stem;
});

async function handleSubmit() {
	if (!name.trim() || !appState.system || !appState.openFilePath) return;
	const stem =
		appState.openFilePath.split('/').pop()?.replace(/\.md$/, '') ?? '';
	if (name.trim() === stem) {
		open = false;
		return;
	}
	loading = true;
	error = '';
	try {
		const newPath = await renameNote(
			appState.system.path,
			appState.openFilePath,
			name.trim(),
		);
		const content = await fileRead(newPath);
		openFile(newPath, content);
		refreshFileTree();
		open = false;
	} catch (e) {
		error = String(e);
	} finally {
		loading = false;
	}
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter') {
		e.preventDefault();
		handleSubmit();
	}
}
</script>

<Dialog bind:open title="Rename Note" contentClass="rename-note-dialog">
  <div class="content">
    <Input
      bind:value={name}
      placeholder="Note name..."
      onkeydown={handleKeydown}
      onfocus={(e) => (e.target as HTMLInputElement).select()}
      autofocus
    />
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <div class="actions">
      <Button variant="secondary" onclick={() => (open = false)}>Cancel</Button>
      <Button variant="primary" onclick={handleSubmit} disabled={!name.trim() || loading}>
        {loading ? 'Renaming…' : 'Rename'}
      </Button>
    </div>
  </div>
</Dialog>

<style>
  .content {
    padding: 24px;
    width: 360px;
  }

  .error {
    margin: 8px 0 0 0;
    font-size: 12px;
    color: var(--error);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
</style>
