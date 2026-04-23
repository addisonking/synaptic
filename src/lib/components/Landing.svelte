<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import type { SystemInfo } from '$lib/types';
  import { Dialog, Button, Input } from '$lib/components/ui';

  interface Props {
    recents: SystemInfo[];
    onOpenVault: (path: string) => void;
    onCreateVault: (parent: string, name: string) => void;
  }

  let { recents, onOpenVault, onCreateVault }: Props = $props();
  let createName = $state('');
  let createParent = $state('');
  let showCreate = $state(false);
  let createError = $state('');

  async function handleOpenExisting() {
    const selected = await open({ directory: true });
    if (selected && typeof selected === 'string') {
      onOpenVault(selected);
    }
  }

  async function handleCreateSubmit() {
    if (!createName.trim() || !createParent.trim()) return;
    createError = '';
    try {
      await onCreateVault(createParent, createName.trim());
      createName = '';
      createParent = '';
      showCreate = false;
    } catch (e: any) {
      createError = e?.message || String(e);
    }
  }

  async function pickCreateParent() {
    const selected = await open({ directory: true });
    if (selected && typeof selected === 'string') {
      createParent = selected;
    }
  }
</script>

<div class="landing">
  <div class="landing-content">
    <h1 class="logo">Synaptic</h1>

    <div class="actions">
      <Button variant="primary" size="lg" onclick={handleOpenExisting}>
        Open Existing Vault
      </Button>
      <Button variant="secondary" size="lg" onclick={() => (showCreate = true)}>
        Create New Vault
      </Button>
    </div>

    {#if recents.length > 0}
      <div class="recents">
        <h2>Recent Vaults</h2>
        <ul>
          {#each recents as vault}
            <li>
              <button class="recent-item" onclick={() => onOpenVault(vault.path)}>
                <span class="recent-name">{vault.name}</span>
                <span class="recent-path">{vault.path}</span>
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    <Dialog bind:open={showCreate} title="Create New Vault" contentClass="create-dialog">
      <div class="create-form">
        <div class="form-row">
          <label for="vault-name">Name</label>
          <Input id="vault-name" bind:value={createName} placeholder="My Notes" />
        </div>
        <div class="form-row">
          <label for="vault-location">Location</label>
          <div class="path-row">
            <Input id="vault-location" bind:value={createParent} placeholder="/path/to/parent" readonly />
            <Button variant="default" size="sm" onclick={pickCreateParent}>Browse</Button>
          </div>
        </div>
        {#if createError}
          <div class="error-text">{createError}</div>
        {/if}
        <div class="modal-actions">
          <Button
            variant="secondary"
            onclick={() => { showCreate = false; createError = ''; }}
          >
            Cancel
          </Button>
          <Button
            variant="primary"
            onclick={handleCreateSubmit}
            disabled={!createName.trim() || !createParent.trim()}
          >
            Create
          </Button>
        </div>
      </div>
    </Dialog>
  </div>
</div>

<style>
  .landing {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--bg);
  }
  .landing-content {
    text-align: center;
    max-width: 480px;
    width: 100%;
    padding: 40px;
  }
  .logo {
    font-size: 32px;
    font-weight: 400;
    color: var(--text-active);
    letter-spacing: 2px;
    margin-bottom: 8px;
  }
  .actions {
    display: flex;
    gap: 12px;
    justify-content: center;
    margin-bottom: 48px;
  }
  .recents {
    text-align: left;
  }
  .recents h2 {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--muted-2);
    margin-bottom: 12px;
    font-weight: 400;
  }
  .recents ul {
    list-style: none;
  }
  .recent-item {
    display: flex;
    flex-direction: column;
    width: 100%;
    text-align: left;
    padding: 10px 12px;
    background: var(--surface-1);
    border: 1px solid transparent;
    margin-bottom: 4px;
    transition: background 80ms, border-color 80ms;
    cursor: pointer;
    font-family: var(--font);
    font-size: 13px;
    color: inherit;
  }
  .recent-item:hover {
    background: var(--surface-2);
    border-color: var(--surface-3);
  }
  .recent-name {
    color: var(--text-bright);
    font-size: 13px;
  }
  .recent-path {
    color: var(--muted-2);
    font-size: 11px;
    margin-top: 2px;
  }
  .create-form {
    padding: 0 24px 24px;
    width: 400px;
    text-align: left;
  }
  .form-row {
    margin-bottom: 16px;
  }
  .form-row label {
    display: block;
    font-size: 11px;
    color: var(--muted-2);
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: 6px;
  }
  .path-row {
    display: flex;
    gap: 8px;
  }
  .path-row :global(.ui-input) {
    flex: 1;
  }
  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 12px;
  }
  .error-text {
    color: var(--error);
    font-size: 12px;
    margin-bottom: 12px;
    word-break: break-word;
  }
</style>
