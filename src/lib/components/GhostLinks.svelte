<script lang="ts">
  import { appState } from '$lib/store.svelte';
  import { scanGhostLinks, previewGhostNoteStream, createGhostNotes, fileRead } from '$lib/api';
  import type { GhostLink, GhostNotePreview, GhostSource } from '$lib/types';
  import { X, Sparkles, Check, XCircle, RotateCcw } from 'lucide-svelte';
  import { Channel } from '@tauri-apps/api/core';
  import { Dialog, Button, Separator } from '$lib/components/ui';

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
  let open = $state(true);

  let ghosts = $state<GhostLink[]>([]);
  let selectedIdx = $state(0);
  let isScanning = $state(false);
  let scanError = $state<string | null>(null);

  let previews = $state<Record<string, string>>({});
  let generating = $state<Record<string, boolean>>({});
  let generatingStatus = $state<Record<string, string>>({});
  let statuses = $state<Record<string, 'pending' | 'approved' | 'rejected'>>({});
  let genErrors = $state<Record<string, string>>({});

  let isCreating = $state(false);
  let createError = $state<string | null>(null);

  $effect(() => {
    if (!open) {
      onClose();
    }
  });

  async function doScan() {
    if (!appState.system) return;
    isScanning = true;
    scanError = null;
    try {
      const result = await scanGhostLinks(appState.system.path);
      ghosts = result;
      selectedIdx = 0;
      previews = {};
      generating = {};
      statuses = {};
      genErrors = {};
      appState.ghostLinkCount = result.length;
    } catch (e) {
      scanError = String(e);
    } finally {
      isScanning = false;
    }
  }

  async function doGenerate(target: string, sources: GhostSource[]) {
    if (!appState.system) return;
    generating[target] = true;
    genErrors[target] = '';
    previews[target] = '';
    generatingStatus[target] = 'Starting...';
    try {
      const channel = new Channel<{ kind: string; data: string }>();
      channel.onmessage = (msg) => {
        if (msg.kind === 'status') {
          generatingStatus[target] = msg.data;
        } else if (msg.kind === 'chunk') {
          previews[target] += msg.data;
        } else if (msg.kind === 'done') {
          generating[target] = false;
          generatingStatus[target] = '';
          statuses[target] = 'pending';
        } else if (msg.kind === 'error') {
          genErrors[target] = msg.data;
          generating[target] = false;
          generatingStatus[target] = '';
        }
      };
      await previewGhostNoteStream(appState.system.path, target, sources, channel);
    } catch (e) {
      genErrors[target] = String(e);
      generating[target] = false;
      generatingStatus[target] = '';
    }
  }

  function approve(target: string) {
    statuses[target] = 'approved';
  }

  function reject(target: string) {
    statuses[target] = 'rejected';
  }

  async function doCreate() {
    if (!appState.system) return;
    const approved: GhostNotePreview[] = [];
    for (const g of ghosts) {
      if (statuses[g.target] === 'approved' && previews[g.target]) {
        approved.push({ target: g.target, content: previews[g.target] });
      }
    }
    if (approved.length === 0) return;
    isCreating = true;
    createError = null;
    try {
      await createGhostNotes(appState.system.path, approved);
      ghosts = ghosts.filter((g) => statuses[g.target] !== 'approved');
      appState.ghostLinkCount = ghosts.length;
      selectedIdx = Math.min(selectedIdx, ghosts.length - 1);
    } catch (e) {
      createError = String(e);
    } finally {
      isCreating = false;
    }
  }

  function approveAll() {
    for (const g of ghosts) {
      if (previews[g.target]) {
        statuses[g.target] = 'approved';
      }
    }
  }

  function rejectAll() {
    for (const g of ghosts) {
      statuses[g.target] = 'rejected';
    }
  }

  function statusColor(target: string): string {
    const s = statuses[target];
    if (s === 'approved') return '#44cc44';
    if (s === 'rejected') return '#cc4444';
    if (previews[target]) return '#4444cc';
    return '#666666';
  }

  function approvedCount(): number {
    return ghosts.filter((g) => statuses[g.target] === 'approved').length;
  }

  function collectTags(sources: GhostSource[]): string[] {
    const seen = new Set<string>();
    const result: string[] = [];
    for (const src of sources) {
      for (const tag of src.tags) {
        if (!seen.has(tag)) {
          seen.add(tag);
          result.push(tag);
        }
      }
    }
    return result;
  }

  $effect(() => {
    doScan();
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<Dialog bind:open contentClass="ghost-dialog" showClose={false}>
  <div class="ghost-layout">
    <div class="modal-header">
      <div class="header-left">
        <h2>Ghost Links</h2>
        <span class="meta">{ghosts.length} unresolved · {approvedCount()} approved</span>
      </div>
      <div class="header-actions">
        <Button variant="ghost" size="icon" onclick={doScan} disabled={isScanning} title="Rescan">
          <RotateCcw size={14} />
        </Button>
        <Button variant="ghost" size="icon" onclick={() => (open = false)} title="Close">
          <X size={14} />
        </Button>
      </div>
    </div>

    {#if isScanning}
      <div class="loading">Scanning vault for ghost links…</div>
    {:else if scanError}
      <div class="loading error">{scanError}</div>
    {:else if ghosts.length === 0}
      <div class="loading">No ghost links found. All wiki-links resolve to existing notes.</div>
    {:else}
      <div class="body">
        <div class="sidebar">
          {#each ghosts as ghost, i}
            <button
              class="ghost-row"
              class:selected={i === selectedIdx}
              onclick={() => (selectedIdx = i)}
            >
              <span class="status-dot" style="color: {statusColor(ghost.target)}">●</span>
              <div class="ghost-info">
                <div class="ghost-name">{ghost.target}</div>
                <div class="ghost-meta">
                  {ghost.sources.length} reference{ghost.sources.length === 1 ? '' : 's'}
                  · {ghost.sources[0]?.note_name ?? ''}
                </div>
              </div>
            </button>
          {/each}
        </div>

        <Separator orientation="vertical" />

        <div class="detail">
          {#if ghosts[selectedIdx]}
            {@const ghost = ghosts[selectedIdx]}
            <div class="detail-header">
              <h3>{ghost.target}</h3>
              <div class="detail-actions">
                {#if !previews[ghost.target] && !generating[ghost.target]}
                  <Button variant="default" size="sm" onclick={() => doGenerate(ghost.target, ghost.sources)}>
                    <Sparkles size={13} />
                    Generate
                  </Button>
                {:else if generating[ghost.target]}
                  <span class="generating">{generatingStatus[ghost.target] || 'Generating...'}</span>
                {:else}
                  <Button variant="default" size="sm" onclick={() => approve(ghost.target)}>
                    <Check size={13} />
                    Approve
                  </Button>
                  <Button variant="destructive" size="sm" onclick={() => reject(ghost.target)}>
                    <XCircle size={13} />
                    Reject
                  </Button>
                {/if}
              </div>
            </div>

            {#if genErrors[ghost.target]}
              <div class="error-box">{genErrors[ghost.target]}</div>
            {/if}

            <div class="sections">
              {#if collectTags(ghost.sources).length > 0}
                <div class="section">
                  <div class="section-label">Tags from Sources</div>
                  <div class="tag-list">
                    {#each collectTags(ghost.sources) as tag}
                      <span class="tag-pill">{tag}</span>
                    {/each}
                  </div>
                </div>
              {/if}

              <div class="section">
                <div class="section-label">Source Context</div>
                <div class="context-list">
                  {#each ghost.sources as src}
                    <div class="context-item">
                      <div class="context-meta">{src.note_name} · line {src.line}</div>
                      <pre class="context-text">{src.context}</pre>
                    </div>
                  {/each}
                </div>
              </div>

              {#if previews[ghost.target] !== undefined}
                <div class="section">
                  <div class="section-label">Preview</div>
                  <textarea
                    class="preview-editor"
                    bind:value={previews[ghost.target]}
                    spellcheck="false"
                    readonly={generating[ghost.target]}
                  ></textarea>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <div class="footer">
        <div class="footer-left">
          <Button variant="secondary" size="sm" onclick={approveAll}>Approve All</Button>
          <Button variant="secondary" size="sm" onclick={rejectAll}>Reject All</Button>
        </div>
        <div class="footer-right">
          {#if createError}
            <span class="footer-error">{createError}</span>
          {/if}
          <Button
            variant="default"
            size="sm"
            onclick={doCreate}
            disabled={isCreating || approvedCount() === 0}
          >
            {#if isCreating}
              Creating…
            {:else}
              Create {approvedCount()} Notes
            {/if}
          </Button>
        </div>
      </div>
    {/if}
  </div>
</Dialog>

<style>
  :global(.ghost-dialog) {
    width: 900px;
    height: 640px;
    max-width: 95vw;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
  }
  .ghost-layout {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .header-left {
    display: flex;
    align-items: baseline;
    gap: 12px;
  }
  .header-left h2 {
    font-size: 14px;
    color: var(--text-bright);
    font-weight: 500;
    margin: 0;
  }
  .meta {
    font-size: 12px;
    color: var(--muted-2);
  }
  .header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .loading {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--muted-2);
    font-size: 13px;
    padding: 40px;
    text-align: center;
  }
  .loading.error {
    color: var(--error);
  }
  .body {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }
  .sidebar {
    width: 260px;
    overflow-y: auto;
    flex-shrink: 0;
    padding: 8px;
  }
  .ghost-row {
    width: 100%;
    text-align: left;
    padding: 8px 10px;
    border: none;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    border-radius: 4px;
    transition: background 80ms;
    font-family: var(--font);
    font-size: 13px;
    color: inherit;
  }
  .ghost-row:hover {
    background: var(--surface-2);
  }
  .ghost-row.selected {
    background: var(--surface-3);
  }
  .status-dot {
    font-size: 10px;
    flex-shrink: 0;
    margin-top: 3px;
  }
  .ghost-info {
    min-width: 0;
  }
  .ghost-name {
    font-size: 12px;
    color: var(--text-bright);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .ghost-meta {
    font-size: 11px;
    color: var(--muted-2);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .detail {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .detail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 16px;
    flex-shrink: 0;
  }
  .detail-header h3 {
    font-size: 14px;
    color: var(--text-bright);
    font-weight: 500;
    margin: 0;
  }
  .detail-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .generating {
    font-size: 11px;
    color: var(--muted-2);
  }
  .error-box {
    background: rgba(204, 68, 68, 0.1);
    border: 1px solid rgba(204, 68, 68, 0.3);
    color: #cc8888;
    font-size: 12px;
    padding: 10px 12px;
    margin-bottom: 12px;
    border-radius: 4px;
  }
  .sections {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-label {
    font-size: 10px;
    color: var(--muted-2);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .context-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .context-item {
    background: var(--bg);
    border: 1px solid var(--border);
    padding: 10px 12px;
    border-radius: 4px;
  }
  .context-meta {
    font-size: 11px;
    color: var(--muted-2);
    margin-bottom: 6px;
  }
  .context-text {
    font-family: 'Geist Mono', monospace;
    font-size: 11px;
    color: var(--text);
    line-height: 1.5;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
  }
  .preview-editor {
    width: 100%;
    flex: 1;
    min-height: 200px;
    background: var(--bg);
    border: 1px solid var(--border);
    padding: 12px;
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
    color: var(--text);
    line-height: 1.6;
    resize: vertical;
    outline: none;
  }
  .preview-editor:focus {
    border-color: var(--surface-3);
  }
  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-top: 1px solid var(--border);
    flex-shrink: 0;
  }
  .footer-left,
  .footer-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .footer-error {
    font-size: 11px;
    color: var(--error);
  }
  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .tag-pill {
    display: inline-flex;
    align-items: center;
    padding: 3px 10px;
    font-size: 11px;
    color: #8888cc;
    background: rgba(136, 136, 204, 0.08);
    border: 1px solid rgba(136, 136, 204, 0.2);
    border-radius: 4px;
    font-family: var(--font);
  }
</style>
