<script lang="ts">
import { fileRead, generateNote } from '$lib/api';
import { Button, Dialog, Input } from '$lib/components/ui';
import { appState, openFile, refreshFileTree } from '$lib/store.svelte';

interface Props {
	onClose: () => void;
}

let { onClose }: Props = $props();
let open = $state(true);
let name = $state('');
let tagsRaw = $state('');
let loading = $state(false);
let error = $state('');

$effect(() => {
	if (!open) onClose();
});

async function handleSubmit() {
	if (!name.trim() || !appState.system) return;
	loading = true;
	error = '';
	try {
		const tags = tagsRaw
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);
		const path = await generateNote(name.trim(), tags, appState.system.path);
		const content = await fileRead(path);
		openFile(path, content);
		refreshFileTree();
		open = false;
	} catch (e) {
		error = String(e);
	} finally {
		loading = false;
	}
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter' && !loading) {
		e.preventDefault();
		handleSubmit();
	}
}
</script>

<Dialog bind:open title="Generate Note" contentClass="generate-note-dialog">
  <div class="content">
    <Input
      bind:value={name}
      placeholder="Note name..."
      onkeydown={handleKeydown}
      disabled={loading}
      autofocus
    />
    <Input
      bind:value={tagsRaw}
      placeholder="Tags (comma-separated)..."
      onkeydown={handleKeydown}
      disabled={loading}
    />
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <div class="actions">
      <Button variant="secondary" onclick={() => (open = false)} disabled={loading}>Cancel</Button>
      <Button variant="primary" onclick={handleSubmit} disabled={!name.trim() || loading}>
        {loading ? 'Generating…' : 'Generate'}
      </Button>
    </div>
  </div>
</Dialog>

<style>
  .content {
    padding: 24px;
    width: 400px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 6px;
  }
  .error {
    margin: 0;
    font-size: 12px;
    color: var(--error, #e55);
  }
</style>
