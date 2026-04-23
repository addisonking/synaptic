<script lang="ts">
  import { appState, openFile } from '$lib/store.svelte';
  import { fileCreate, fileRead } from '$lib/api';
  import { Dialog, Button, Input } from '$lib/components/ui';

  interface Props {
    onClose: () => void;
    initialName?: string;
  }

  let { onClose, initialName = '' }: Props = $props();
  let name = $state('');

  $effect(() => {
    name = initialName;
  });
  let open = $state(true);

  $effect(() => {
    if (!open) {
      onClose();
    }
  });

  async function handleSubmit() {
    if (!name.trim() || !appState.system) return;
    const notesDir = appState.system.path + '/notes';
    const newPath = notesDir + '/' + name.trim().replace(/\.md$/, '') + '.md';
    await fileCreate(newPath);
    const content = await fileRead(newPath);
    openFile(newPath, content);
    open = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSubmit();
    }
  }
</script>

<Dialog bind:open title="New Note" contentClass="new-note-dialog">
  <div class="content">
    <Input
      bind:value={name}
      placeholder="Note name..."
      onkeydown={handleKeydown}
      autofocus
    />
    <div class="actions">
      <Button variant="secondary" onclick={() => (open = false)}>Cancel</Button>
      <Button variant="primary" onclick={handleSubmit} disabled={!name.trim()}>
        Create
      </Button>
    </div>
  </div>
</Dialog>

<style>
  .content {
    padding: 24px;
    width: 360px;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
</style>
