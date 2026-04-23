<script lang="ts">
  import { onMount } from 'svelte';
  import { appState, openFile } from '$lib/store.svelte';
  import { getBacklinks, findNote, fileRead } from '$lib/api';
  import { renderMarkdown, parseFrontmatter } from '$lib/markdown';
  import type { BacklinkInfo } from '$lib/types';
  import 'katex/dist/katex.min.css';
  import 'highlight.js/styles/github-dark.min.css';
  import { ChevronRight, ChevronDown } from 'lucide-svelte';

  let renderedHtml = $state('');
  let tags = $state<string[]>([]);
  let backlinks = $state<BacklinkInfo[]>([]);
  let backlinksCollapsed = $state(true);
  let previewEl = $state<HTMLDivElement | null>(null);

  let lastContent = $state('');
  let pollInterval: ReturnType<typeof setInterval>;
  let currentPath: string | null = null;
  let totalLines = $state(1);
  let scrollRaf: number | null = null;

  function getTotalLines(content: string) {
    return content.split('\n').length;
  }

  function scrollPreviewToLine(line: number) {
    if (!previewEl || totalLines <= 1) return;
    const ratio = (line - 1) / (totalLines - 1);
    const target = ratio * previewEl.scrollHeight - previewEl.clientHeight / 2;
    const clamped = Math.max(0, Math.min(previewEl.scrollHeight - previewEl.clientHeight, target));

    if (scrollRaf) cancelAnimationFrame(scrollRaf);
    scrollRaf = requestAnimationFrame(() => {
      if (previewEl) previewEl.scrollTo({ top: clamped, behavior: 'smooth' });
      scrollRaf = null;
    });
  }

  async function render() {
    if (!appState.system || !appState.openFilePath) return;
    try {
      const content = await fileRead(appState.openFilePath);
      lastContent = content;
      totalLines = getTotalLines(content);
      const fm = parseFrontmatter(content);
      tags = fm.tags;
      renderedHtml = renderMarkdown(content);

      const noteName = appState.openFilePath.split('/').pop()?.replace('.md', '') || '';
      backlinks = await getBacklinks(appState.system.path, noteName);
    } catch (e) {
      renderedHtml = `<pre style="color:#cc4444">${String(e)}</pre>`;
    }
  }

  async function checkForChanges() {
    if (!appState.openFilePath) return;
    try {
      const content = await fileRead(appState.openFilePath);
      if (content !== lastContent) {
        lastContent = content;
        totalLines = getTotalLines(content);
        const fm = parseFrontmatter(content);
        tags = fm.tags;
        renderedHtml = renderMarkdown(content);
      }
    } catch {
      // ignore
    }
  }

  $effect(() => {
    const path = appState.openFilePath;
    if (path && path !== currentPath) {
      currentPath = path;
      lastContent = '';
      totalLines = 1;
      renderedHtml = '';
      tags = [];
      backlinks = [];
      appState.cursorLineActive = false;
      if (previewEl) previewEl.scrollTop = 0;
      render();
    }
  });

  $effect(() => {
    const line = appState.cursorLine;
    if (appState.cursorLineActive && previewEl && totalLines > 1) {
      scrollPreviewToLine(line);
    }
  });

  onMount(() => {
    if (appState.openFilePath) {
      currentPath = appState.openFilePath;
      render();
    }
    pollInterval = setInterval(checkForChanges, 500);
    return () => clearInterval(pollInterval);
  });

  async function onPreviewClick(e: MouseEvent) {
    const target = e.target as Element;
    const link = target.closest('a');
    if (!link || !appState.system) return;

    const href = link.getAttribute('href') || '';
    if (href.startsWith('wiki://')) {
      e.preventDefault();
      const noteName = decodeURIComponent(href.slice(7));
      const notePath = await findNote(appState.system.path, noteName);
      if (notePath) {
        const content = await fileRead(notePath);
        openFile(notePath, content);
      }
    }
  }
</script>

<div class="editor-wrap">
  {#if !appState.openFilePath}
    <div class="empty-state">
      <span>no note open</span>
      <span class="hint">⌘P to open or search · ⌘N to create</span>
    </div>
  {:else}
    <div class="preview" bind:this={previewEl} onclick={onPreviewClick}>
      {#if tags.length > 0}
        <div class="tag-list">
          {#each tags as tag}
            <span class="tag-pill">{tag}</span>
          {/each}
        </div>
      {/if}
      {#if renderedHtml}
        {@html renderedHtml}
      {:else}
        <div class="compiling">loading preview...</div>
      {/if}
    </div>
    {#if backlinks.length > 0}
      <div class="backlinks" class:collapsed={backlinksCollapsed}>
        <button class="backlinks-label" onclick={() => backlinksCollapsed = !backlinksCollapsed}>
          <span class="backlinks-chevron">
            {#if backlinksCollapsed}
              <ChevronRight size={10} />
            {:else}
              <ChevronDown size={10} />
            {/if}
          </span>
          backlinks
          <span class="backlinks-count">{backlinks.length}</span>
        </button>
        {#if !backlinksCollapsed}
          {#each backlinks as bl}
            <div class="backlink-item" onclick={async () => {
              const content = await fileRead(bl.note_path);
              openFile(bl.note_path, content);
            }}>
              {bl.note_name}
            </div>
          {/each}
        {/if}
      </div>
    {/if}
  {/if}
</div>

<style>
  .editor-wrap {
    height: 100%;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: #000;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: #333333;
    font-size: 13px;
  }

  .hint {
    font-size: 12px;
    color: #222222;
  }

  .preview {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 32px 40px;
    background: #000;
    line-height: 1.6;
    word-wrap: break-word;
    overflow-wrap: break-word;
  }

  /* Markdown content styling */
  .preview :global(h1) {
    font-size: 20px;
    font-weight: 600;
    color: #ffffff;
    margin: 24px 0 12px;
    padding-bottom: 8px;
    border-bottom: 1px solid #1a1a1a;
  }

  .preview :global(h2) {
    font-size: 16px;
    font-weight: 600;
    color: #eeeeee;
    margin: 20px 0 10px;
    padding-bottom: 6px;
    border-bottom: 1px solid #111111;
  }

  .preview :global(h3) {
    font-size: 14px;
    font-weight: 600;
    color: #dddddd;
    margin: 16px 0 8px;
  }

  .preview :global(p) {
    color: #aaaaaa;
    margin: 8px 0;
    font-size: 13px;
  }

  .preview :global(a) {
    color: #888888;
    text-decoration: underline;
    text-underline-offset: 2px;
    cursor: pointer;
  }

  .preview :global(a:hover) {
    color: #cccccc;
  }

  .preview :global(ul),
  .preview :global(ol) {
    margin: 8px 0;
    padding-left: 24px;
    color: #aaaaaa;
    font-size: 13px;
  }

  .preview :global(li) {
    margin: 4px 0;
  }

  .preview :global(pre) {
    background: #0a0a0a;
    border: 1px solid #1a1a1a;
    padding: 12px;
    margin: 12px 0;
    overflow-x: auto;
  }

  .preview :global(code) {
    font-family: 'Geist Mono', monospace;
    font-size: 12px;
  }

  .preview :global(pre code) {
    color: #cccccc;
    background: transparent;
    padding: 0;
  }

  .preview :global(p code) {
    background: #111111;
    padding: 2px 6px;
    color: #cccccc;
    border: 1px solid #1a1a1a;
  }

  .preview :global(blockquote) {
    border-left: 2px solid #333333;
    margin: 12px 0;
    padding-left: 16px;
    color: #888888;
  }

  .preview :global(hr) {
    border: none;
    border-top: 1px solid #1a1a1a;
    margin: 20px 0;
  }

  .preview :global(table) {
    width: 100%;
    border-collapse: collapse;
    margin: 12px 0;
    font-size: 12px;
  }

  .preview :global(th),
  .preview :global(td) {
    border: 1px solid #1a1a1a;
    padding: 8px 12px;
    text-align: left;
  }

  .preview :global(th) {
    background: #0a0a0a;
    color: #eeeeee;
    font-weight: 600;
  }

  .preview :global(td) {
    color: #aaaaaa;
  }

  .preview :global(img) {
    max-width: 100%;
    height: auto;
  }

  .preview :global(.katex) {
    color: #cccccc;
    font-size: 1em;
  }

  .preview :global(.katex-display) {
    margin: 12px 0;
    overflow-x: auto;
  }

  .compiling {
    color: #333333;
    font-size: 11px;
    padding: 32px;
    text-align: center;
  }

  .backlinks {
    border-top: 1px solid #0f0f0f;
    padding: 12px 32px 16px;
    flex-shrink: 0;
  }

  .backlinks.collapsed {
    padding-bottom: 12px;
  }

  .backlinks-label {
    font-size: 10px;
    color: #333333;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 6px;
    display: flex;
    align-items: center;
    gap: 6px;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    width: 100%;
    transition: color 80ms;
  }

  .backlinks-label:hover {
    color: #555555;
  }

  .backlinks-chevron {
    color: #333333;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 10px;
  }

  .backlinks-count {
    color: #222222;
    font-size: 10px;
  }

  .backlink-item {
    font-size: 12px;
    color: #444444;
    cursor: pointer;
    padding: 3px 0;
    transition: color 80ms;
  }

  .backlink-item:hover {
    color: #aaaaaa;
  }

  .tag-list {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 16px;
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
