<script lang="ts">
import { Download, X } from 'lucide-svelte';
import { Button, Dialog } from '$lib/components/ui';
import { downloadAndInstallUpdate, updater } from '$lib/updater.svelte';

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

async function handleInstall() {
	await downloadAndInstallUpdate();
}

function handleLater() {
	open = false;
}
</script>

<Dialog bind:open title="Update Available" contentClass="update-dialog" showClose={false}>
  <div class="update-prompt">
    <p class="version-line">
      A new version <strong>v{updater.version}</strong> is available.
      You are currently on v{updater.appVersion}.
    </p>
    {#if updater.body}
      <div class="update-notes">
        <span class="notes-label">What’s new</span>
        <pre>{updater.body}</pre>
      </div>
    {/if}
    <div class="actions">
      <Button variant="default" size="sm" onclick={handleLater}>
        <X size={14} /> Later
      </Button>
      <Button variant="primary" size="sm" onclick={handleInstall} disabled={updater.status === 'downloading' || updater.status === 'installing'}>
        {#if updater.status === 'downloading'}
          <Download size={14} /> Downloading… {Math.round(updater.progress)}%
        {:else if updater.status === 'installing'}
          Installing…
        {:else}
          <Download size={14} /> Install Now
        {/if}
      </Button>
    </div>
    {#if updater.error}
      <span class="error">{updater.error}</span>
    {/if}
  </div>
</Dialog>

<style>
  .update-prompt {
    padding: 20px;
    width: 400px;
  }
  .version-line {
    font-size: 13px;
    color: var(--text);
    margin: 0 0 12px;
    line-height: 1.5;
  }
  .update-notes {
    margin-bottom: 16px;
  }
  .notes-label {
    display: block;
    font-size: 11px;
    color: var(--text-bright);
    margin-bottom: 6px;
  }
  .update-notes pre {
    font-size: 12px;
    color: var(--muted-2);
    background: var(--surface-2);
    padding: 10px 12px;
    border-radius: 4px;
    max-height: 160px;
    overflow-y: auto;
    white-space: pre-wrap;
    line-height: 1.4;
    margin: 0;
  }
  .actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .error {
    display: block;
    font-size: 11px;
    color: var(--error);
    margin-top: 10px;
  }
</style>
