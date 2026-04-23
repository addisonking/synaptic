<script lang="ts">
import { fileDelete } from '$lib/api';
import { appState, goBack, goForward } from '$lib/store.svelte';

interface Props {
	onFindOrCreate: () => void;
	onNewNote: () => void;
	onBlurTerminal?: () => void;
}

let { onFindOrCreate, onNewNote, onBlurTerminal }: Props = $props();
let deleteClicks = $state(0);

async function handleDelete() {
	if (!appState.openFilePath) return;
	deleteClicks++;
	if (deleteClicks >= 2) {
		await fileDelete(appState.openFilePath);
		appState.openFilePath = null;
		appState.openFileContent = '';
		deleteClicks = 0;
	}
	setTimeout(() => (deleteClicks = 0), 2000);
}
</script>

<div class="titlebar">
  <div class="left">
    <button class="nav-btn" onclick={goBack} disabled={appState.historyIdx <= 0} title="Back">
      &lt;
    </button>
    <button class="nav-btn" onclick={goForward} disabled={appState.historyIdx >= appState.history.length - 1} title="Forward">
      &gt;
    </button>
    <span class="breadcrumb">
      {#if appState.openFilePath}
        <span class="filename">{appState.openFilePath.split('/').pop()?.replace(/\.md$/, '')}</span>
      {:else if appState.system}
        {appState.system.name}
      {/if}
    </span>
  </div>

  <div class="center">
    <button class="action-btn" onclick={onFindOrCreate} title="Find or Create (⌘P)">
      Find <span class="keyhint">⌘P</span>
    </button>
    <button class="action-btn" onclick={onNewNote} title="New Note (⌘N)">
      New <span class="keyhint">⌘N</span>
    </button>
    <button class="action-btn" onclick={() => { onBlurTerminal?.(); appState.showGraph = true; }} title="Graph (⌘G)">
      Graph <span class="keyhint">⌘G</span>
    </button>
    {#if appState.ghostLinkCount > 0}
      <button class="action-btn ghost-btn" onclick={() => { onBlurTerminal?.(); appState.showGhostLinks = true; }} title="Ghost Links (⌘⇧G)">
        Ghosts <span class="ghost-badge">{appState.ghostLinkCount}</span>
      </button>
    {/if}
  </div>

  <div class="right">
    {#if appState.openFilePath}
      <button
        class="delete-btn"
        onclick={handleDelete}
        class:confirm={deleteClicks === 1}
        title="Delete note (click twice)"
      >
        {deleteClicks === 1 ? 'Click to confirm' : 'Delete'}
      </button>
    {/if}
    <button class="action-btn" onclick={() => { onBlurTerminal?.(); appState.showSettings = true; }} title="Settings (⌘,)">
      Settings
    </button>
    <button class="action-btn" onclick={() => { onBlurTerminal?.(); appState.showHelp = true; }} title="Help (?)">
      Help
    </button>
  </div>
</div>

<style>
  .titlebar {
    height: 40px;
    background: var(--bg);
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    padding: 0 12px;
    flex-shrink: 0;
    -webkit-user-select: none;
    user-select: none;
  }

  .left, .center, .right {
    display: flex;
    align-items: center;
    gap: 4px;
    flex: 1;
    min-width: 0;
  }

  .left {
    justify-content: flex-start;
  }

  .center {
    justify-content: center;
  }

  .right {
    justify-content: flex-end;
  }

  .nav-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text);
    font-size: 12px;
    border: 1px solid transparent;
    transition: background 80ms;
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--surface-1);
    border-color: var(--surface-2);
  }

  .nav-btn:disabled {
    color: var(--muted-1);
    cursor: not-allowed;
  }

  .breadcrumb {
    color: var(--text);
    font-size: 12px;
    margin-left: 8px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .filename {
    color: var(--text-bright);
  }

  .action-btn {
    padding: 4px 10px;
    font-size: 12px;
    color: var(--text);
    border: 1px solid transparent;
    transition: background 80ms, border-color 80ms;
  }

  .action-btn:hover {
    background: var(--surface-1);
    border-color: var(--surface-2);
  }

  .keyhint {
    color: var(--muted-2);
    font-size: 10px;
    margin-left: 4px;
    font-weight: 400;
  }

  .delete-btn {
    padding: 4px 10px;
    font-size: 11px;
    color: var(--muted-2);
    border: 1px solid transparent;
    transition: all 80ms;
  }

  .delete-btn:hover, .delete-btn.confirm {
    color: var(--error);
    border-color: var(--error);
    background: rgba(204, 68, 68, 0.1);
  }

  .ghost-btn {
    position: relative;
  }

  .ghost-badge {
    display: inline-block;
    background: #3a2a1a;
    color: #ccaa44;
    font-size: 10px;
    padding: 1px 5px;
    border-radius: 3px;
    margin-left: 4px;
    font-weight: 500;
  }

</style>
