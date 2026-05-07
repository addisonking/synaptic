import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { fileTree, getSyncState } from './api';
import type { FileNode, SearchResult, SyncState, SystemInfo } from './types';

// Global application state using Svelte 5 runes
export const appState = $state({
	system: null as SystemInfo | null,
	ollamaAvailable: true,
	recentSystems: [] as SystemInfo[],
	fileTree: [] as FileNode[],
	openFilePath: null as string | null,
	openFileContent: '',
	isDirty: false,
	saveStatus: 'saved' as 'saved' | 'saving' | 'unsaved',
	history: [] as string[],
	historyIdx: -1,
	searchQuery: '',
	searchResults: [] as SearchResult[],
	isSearching: false,
	indexVersion: 0,
	zoom: 100,
	showGraph: false,
	showHelp: false,
	showSettings: false,
	sidebarVisible: true,
	cursorLine: 1,
	cursorLineActive: false,
	syncStatus: {
		syncing: false,
		last_error: null,
		last_sync: null,
	} as SyncState,
});

export function setSystem(system: SystemInfo | null) {
	appState.system = system;
}

export function setRecentSystems(systems: SystemInfo[]) {
	appState.recentSystems = systems;
}

export function setFileTree(tree: FileNode[]) {
	appState.fileTree = tree;
}

export async function refreshFileTree(): Promise<FileNode[]> {
	if (!appState.system) return [];
	const tree = await fileTree(appState.system.path);
	appState.fileTree = tree;
	return tree;
}

export function openFile(path: string, content: string) {
	if (appState.openFilePath) {
		// Truncate forward history
		appState.history = appState.history.slice(0, appState.historyIdx + 1);
		// Push new path if different from current
		if (appState.history[appState.historyIdx] !== path) {
			appState.history.push(path);
			appState.historyIdx = appState.history.length - 1;
		}
	} else {
		appState.history = [path];
		appState.historyIdx = 0;
	}
	appState.openFilePath = path;
	appState.openFileContent = content;
	appState.isDirty = false;
	appState.saveStatus = 'saved';
}

export function setFileContent(content: string) {
	appState.openFileContent = content;
	appState.isDirty = true;
	appState.saveStatus = 'unsaved';
}

export function markSaved() {
	appState.isDirty = false;
	appState.saveStatus = 'saved';
}

export function goBack() {
	if (appState.historyIdx > 0) {
		appState.historyIdx--;
		appState.openFilePath = appState.history[appState.historyIdx];
	}
}

export function goForward() {
	if (appState.historyIdx < appState.history.length - 1) {
		appState.historyIdx++;
		appState.openFilePath = appState.history[appState.historyIdx];
	}
}

export function setZoom(zoom: number) {
	appState.zoom = Math.max(50, Math.min(200, zoom));
	localStorage.setItem('synaptic-zoom', String(appState.zoom));
	getCurrentWebview()
		.setZoom(appState.zoom / 100)
		.catch(() => {});
}

export function loadZoom() {
	const saved = localStorage.getItem('synaptic-zoom');
	if (saved) {
		appState.zoom = parseInt(saved, 10);
		getCurrentWebview()
			.setZoom(appState.zoom / 100)
			.catch(() => {});
	}
}

export async function initSyncListener() {
	// Load initial state
	try {
		appState.syncStatus = await getSyncState();
	} catch {
		// ignore
	}

	// Listen for backend sync-status events
	listen<SyncState>('sync-status', (event) => {
		appState.syncStatus = event.payload;
	}).catch(() => {});
}
