import { fileTree } from './api';
import type { FileNode, SearchResult, SystemInfo } from './types';

// Global application state using Svelte 5 runes
export const appState = $state({
	system: null as SystemInfo | null,
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
}

export function loadZoom() {
	const saved = localStorage.getItem('synaptic-zoom');
	if (saved) {
		appState.zoom = parseInt(saved, 10);
	}
}
