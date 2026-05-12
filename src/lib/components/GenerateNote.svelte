<script lang="ts">
import { Channel } from '@tauri-apps/api/core';
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
let instructions = $state('');
let loading = $state(false);
let streamed = $state('');
let error = $state('');
let channel: Channel<string> | null = null;
let previewEl = $state<HTMLPreElement | null>(null);

$effect(() => {
	if (!open) onClose();
});

$effect(() => {
	// auto-scroll preview to bottom
	if (previewEl && streamed) {
		previewEl.scrollTop = previewEl.scrollHeight;
	}
});

async function handleSubmit() {
	if (!name.trim() || !appState.system) return;
	loading = true;
	streamed = '';
	error = '';

	const ch = new Channel<string>();
	ch.onmessage = (chunk) => {
		streamed += chunk;
	};
	channel = ch;

	try {
		const tags = tagsRaw
			.split(',')
			.map((t) => t.trim())
			.filter((t) => t.length > 0);
		const path = await generateNote(
			name.trim(),
			tags,
			appState.system.path,
			instructions.trim(),
			ch,
		);
		const content = await fileRead(path);
		openFile(path, content);
		refreshFileTree();
		open = false;
	} catch (e) {
		error = String(e);
	} finally {
		loading = false;
		channel = null;
	}
}

function handleCancel() {
	if (loading && channel) {
		channel = null;
		loading = false;
		streamed = '';
	}
	open = false;
}

function handleKeydown(e: KeyboardEvent) {
	if (e.key === 'Enter' && !loading) {
		e.preventDefault();
		handleSubmit();
	}
}
</script>

<Dialog bind:open title="Generate Note" contentClass="generate-note-dialog">
  <div class="content" class:streaming={loading || streamed}>
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
    <Input
      bind:value={instructions}
      placeholder="Extra instructions (optional)..."
      onkeydown={handleKeydown}
      disabled={loading}
    />
    {#if loading || streamed}
      <pre class="preview" bind:this={previewEl}>{streamed}</pre>
    {/if}
    {#if error}
      <p class="error">{error}</p>
    {/if}
    <div class="actions">
      <Button variant="secondary" onclick={handleCancel}>
        {loading ? 'Cancel' : 'Cancel'}
      </Button>
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
  .content.streaming {
    width: 560px;
  }
  .preview {
    margin: 0;
    padding: 12px;
    background: var(--surface, #1e1e1e);
    border: 1px solid var(--border, #333);
    border-radius: 6px;
    font-family: var(--font-mono, ui-monospace, monospace);
    font-size: 13px;
    line-height: 1.5;
    color: var(--text, #eee);
    max-height: 280px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-word;
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
