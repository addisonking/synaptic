<script lang="ts">
import { onMount } from 'svelte';
import {
	checkDependencies,
	fileRead,
	fileRename,
	generateNoteTitle,
	scratchCreate,
	systemListRecent,
	systemOpen,
	vaultCreate,
	vaultGetConfig,
} from '$lib/api';
import DependencyCheck from '$lib/components/DependencyCheck.svelte';
import Editor from '$lib/components/Editor.svelte';
import FindOrCreate from '$lib/components/FindOrCreate.svelte';
import Graph from '$lib/components/Graph.svelte';
import KeybindHelp from '$lib/components/KeybindHelp.svelte';
import Landing from '$lib/components/Landing.svelte';
import NewNote from '$lib/components/NewNote.svelte';
import NvimTerminal from '$lib/components/NvimTerminal.svelte';
import RenameNote from '$lib/components/RenameNote.svelte';
import ScratchFinder from '$lib/components/ScratchFinder.svelte';
import SemanticSearch from '$lib/components/SemanticSearch.svelte';
import Settings from '$lib/components/Settings.svelte';
import Titlebar from '$lib/components/Titlebar.svelte';
import {
	appState,
	goBack,
	goForward,
	initSyncListener,
	loadZoom,
	openFile,
	refreshFileTree,
	setZoom,
} from '$lib/store.svelte';
import type { DependencyStatus } from '$lib/types';

let showFindOrCreate = $state(false);
let showNewNote = $state(false);
let showRenameNote = $state(false);
let showSemanticSearch = $state(false);
let showScratchFinder = $state(false);
let newNoteName = $state('');
let ptyId = $state(0);
let nvimTerminalRef: { blur(): void; focus(): void } | undefined = $state();
let previousFilePath: string | null = $state(null);

let depsReady = $state(false);
let depsSatisfied = $state(false);
let depsStatus = $state<DependencyStatus | null>(null);

function blurTerminal() {
	nvimTerminalRef?.blur();
}

function isScratchPath(path: string | null): boolean {
	if (!path) return false;
	return path.includes('/scratch/');
}

function isTimestampName(name: string): boolean {
	return /^\d{4}-\d{2}-\d{2}-\d{2}-\d{2}-\d{2}$/.test(name);
}

async function tryAutoRenameScratch(oldPath: string) {
	if (!appState.system) return;
	if (!isScratchPath(oldPath)) return;

	const filename = oldPath.split('/').pop()?.replace(/\.md$/, '') || '';
	if (!isTimestampName(filename)) return;

	try {
		const newPath = await generateNoteTitle(appState.system.path, oldPath);
		if (newPath && newPath !== oldPath) {
			await fileRename(oldPath, newPath);
			await refreshFileTree();
		}
	} catch {
		// Auto-rename is best-effort; don't disrupt the user if it fails
	}
}

async function createScratchNote() {
	if (!appState.system) return;
	blurTerminal();
	const path = await scratchCreate(appState.system.path);
	const content = await fileRead(path);
	openFile(path, content);
	refreshFileTree();
}

$effect(() => {
	const currentPath = appState.openFilePath;
	if (previousFilePath && previousFilePath !== currentPath) {
		tryAutoRenameScratch(previousFilePath);
	}
	previousFilePath = currentPath;
});

onMount(() => {
	loadZoom();
	initSyncListener();
	checkDependencies()
		.then((status) => {
			depsStatus = status;
			depsReady = true;
			appState.ollamaAvailable = status.ollama_running;
			if (status.neovim_installed) {
				depsSatisfied = true;
				systemListRecent().then((recents) => {
					appState.recentSystems = recents;
				});
			}
		})
		.catch(() => {
			depsReady = true;
		});

	const handleKeydown = (e: KeyboardEvent) => {
		// Disable global shortcuts when no vault is open (landing screen)
		if (!appState.system) return;

		// Always handle Cmd/Ctrl shortcuts even when terminal/input is focused
		if (e.metaKey || e.ctrlKey) {
			if (e.key === 'p' && !e.shiftKey) {
				e.preventDefault();
				blurTerminal();
				showFindOrCreate = true;
				return;
			}
			if (e.key === 'P' || (e.key === 'p' && e.shiftKey)) {
				e.preventDefault();
				blurTerminal();
				showSemanticSearch = true;
				return;
			}
			if (e.key === 'n' && !e.shiftKey) {
				e.preventDefault();
				blurTerminal();
				showNewNote = true;
				return;
			}
			if (e.key === 'r' && !e.shiftKey && appState.openFilePath) {
				e.preventDefault();
				blurTerminal();
				showRenameNote = true;
				return;
			}
			if (e.key === 'N' || (e.key === 'n' && e.shiftKey)) {
				e.preventDefault();
				createScratchNote();
				return;
			}
			if (e.key === 'S' || (e.key === 's' && e.shiftKey)) {
				e.preventDefault();
				blurTerminal();
				showScratchFinder = true;
				return;
			}
			if (e.key === 'g' && !e.shiftKey) {
				e.preventDefault();
				blurTerminal();
				appState.showGraph = !appState.showGraph;
				return;
			}
			if (e.key === '[') {
				e.preventDefault();
				goBack();
				return;
			}
			if (e.key === ']') {
				e.preventDefault();
				goForward();
				return;
			}
			if (e.key === '=' || e.key === '+') {
				e.preventDefault();
				setZoom(appState.zoom + 10);
				return;
			}
			if (e.key === '-') {
				e.preventDefault();
				setZoom(appState.zoom - 10);
				return;
			}
			if (e.key === '0') {
				e.preventDefault();
				setZoom(100);
				return;
			}
			if (e.key === ',') {
				e.preventDefault();
				blurTerminal();
				appState.showSettings = true;
				return;
			}
		}

		if (e.key === 'Escape') {
			const hadDialog =
				showFindOrCreate ||
				showNewNote ||
				showRenameNote ||
				showSemanticSearch ||
				showScratchFinder ||
				appState.showHelp ||
				appState.showGraph ||
				appState.showSettings;
			showFindOrCreate = false;
			showNewNote = false;
			showRenameNote = false;
			showSemanticSearch = false;
			showScratchFinder = false;
			appState.showHelp = false;
			appState.showGraph = false;
			appState.showSettings = false;
			if (hadDialog) {
				nvimTerminalRef?.focus();
			}
			return;
		}

		// Don't fire global shortcuts inside modal overlays
		if (
			showFindOrCreate ||
			showNewNote ||
			showRenameNote ||
			showSemanticSearch ||
			showScratchFinder ||
			appState.showHelp ||
			appState.showSettings
		) {
			return;
		}

		// Only handle remaining shortcuts when not in input
		const target = e.target as HTMLElement;
		if (target?.tagName === 'INPUT' || target?.tagName === 'TEXTAREA') {
			return;
		}

		if (e.key === '?') {
			e.preventDefault();
			blurTerminal();
			appState.showHelp = true;
			return;
		}
	};

	window.addEventListener('keydown', handleKeydown);
	return () => window.removeEventListener('keydown', handleKeydown);
});

async function handleOpenVault(path: string) {
	const status = await checkDependencies();
	appState.ollamaAvailable = status.ollama_running;
	if (!status.neovim_installed) {
		depsStatus = status;
		depsSatisfied = false;
		return;
	}
	const system = await systemOpen(path);
	appState.system = system;
	appState.recentSystems = await systemListRecent();
	refreshFileTree();
	// Try to open last file
	try {
		const config = await vaultGetConfig(system.path);
		if (config.last_file) {
			const content = await fileRead(config.last_file);
			openFile(config.last_file, content);
		}
	} catch {
		// No last file or failed to read — that's fine
	}
}

async function handleCreateVault(parent: string, name: string) {
	const status = await checkDependencies();
	appState.ollamaAvailable = status.ollama_running;
	if (!status.neovim_installed) {
		depsStatus = status;
		depsSatisfied = false;
		return;
	}
	const path = await vaultCreate(parent, name);
	await handleOpenVault(path);
}
</script>

<div class="app">
  {#if !depsReady}
    <div class="loading-screen">
      <p>Checking dependencies…</p>
    </div>
  {:else if !depsSatisfied && depsStatus}
    <DependencyCheck
      initialStatus={depsStatus}
      onResolved={() => {
        depsSatisfied = true;
        systemListRecent().then((recents) => {
          appState.recentSystems = recents;
        });
      }}
    />
  {:else if !appState.system}
    <Landing
      recents={appState.recentSystems}
      onOpenVault={handleOpenVault}
      onCreateVault={handleCreateVault}
    />
  {:else}
    <div class="main-app">
      <Titlebar
        onFindOrCreate={() => { blurTerminal(); showFindOrCreate = true; }}
        onNewNote={() => { blurTerminal(); showNewNote = true; }}
        onNewScratch={createScratchNote}
        onBlurTerminal={blurTerminal}
      />
      <div class="content">
        {#if appState.openFilePath}
          <div class="split-pane">
            <div class="editor-pane">
              <Editor />
            </div>
            <div class="divider"></div>
            <div class="terminal-pane">
              {#key appState.openFilePath}
                <NvimTerminal bind:this={nvimTerminalRef} filePath={appState.openFilePath} ptyId={ptyId} />
              {/key}
            </div>
          </div>
        {:else}
          <div class="empty-state">
            <div class="empty-hint">
              <p>No note open</p>
              <p class="muted">Press <kbd>⌘P</kbd> to find or create a note</p>
              <p class="muted">Press <kbd>⌘N</kbd> to create a new note</p>
              <p class="muted">Press <kbd>⌘⇧N</kbd> for a quick scratch note</p>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

{#if showFindOrCreate}
  <FindOrCreate onClose={() => showFindOrCreate = false} />
{/if}
{#if showNewNote}
  <NewNote onClose={() => showNewNote = false} initialName={newNoteName} />
{/if}
{#if showRenameNote}
  <RenameNote onClose={() => showRenameNote = false} />
{/if}
{#if showSemanticSearch}
  <SemanticSearch onClose={() => showSemanticSearch = false} />
{/if}
{#if showScratchFinder}
  <ScratchFinder onClose={() => showScratchFinder = false} />
{/if}
{#if appState.showSettings}
  <Settings onClose={() => appState.showSettings = false} />
{/if}
{#if appState.showHelp}
  <KeybindHelp onClose={() => appState.showHelp = false} />
{/if}
{#if appState.showGraph}
  <Graph onClose={() => appState.showGraph = false} />
{/if}

<style>
  .app {
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg);
  }

  .loading-screen {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    width: 100%;
    color: var(--muted-2);
    font-size: 13px;
  }

  .main-app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .content {
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .split-pane {
    display: flex;
    height: 100%;
    width: 100%;
  }

  .editor-pane {
    flex: 1;
    overflow: auto;
    min-width: 0;
  }

  .divider {
    width: 1px;
    background: var(--border);
    flex-shrink: 0;
  }

  .terminal-pane {
    flex: 1;
    overflow: hidden;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    width: 100%;
  }

  .empty-hint {
    text-align: center;
    color: var(--text);
  }

  .empty-hint p {
    margin: 8px 0;
  }

  .muted {
    color: var(--muted-3);
    font-size: 12px;
  }

  kbd {
    background: var(--surface-2);
    border: 1px solid var(--surface-3);
    padding: 2px 6px;
    font-family: var(--font);
    font-size: 11px;
    color: var(--text-bright);
  }
</style>
