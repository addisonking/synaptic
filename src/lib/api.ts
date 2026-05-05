import { type Channel, invoke } from '@tauri-apps/api/core';
import type {
	AppSettings,
	BacklinkInfo,
	DependencyStatus,
	FileNode,
	GraphData,
	OllamaHealth,
	ScratchEntry,
	SearchResult,
	SemanticResult,
	SystemInfo,
	TagEntry,
} from './types';

// Vault lifecycle
export function systemOpen(path: string): Promise<SystemInfo> {
	return invoke('system_open', { path });
}

export function systemListRecent(): Promise<SystemInfo[]> {
	return invoke('system_list_recent');
}

export function vaultCreate(parent: string, name: string): Promise<string> {
	return invoke('vault_create', { parent, name });
}

export function vaultSetLastFile(
	systemPath: string,
	filePath: string,
): Promise<void> {
	return invoke('vault_set_last_file', { systemPath, filePath });
}

export function vaultGetConfig(
	systemPath: string,
): Promise<{ last_file?: string }> {
	return invoke('vault_get_config', { systemPath });
}

// File tree
export function fileTree(systemPath: string): Promise<FileNode[]> {
	return invoke('file_tree', { systemPath });
}

// File I/O
export function fileRead(path: string): Promise<string> {
	return invoke('file_read', { path });
}

export function fileWrite(path: string, content: string): Promise<void> {
	return invoke('file_write', { path, content });
}

export function fileCreate(path: string): Promise<void> {
	return invoke('file_create', { path });
}

export function fileDelete(path: string): Promise<void> {
	return invoke('file_delete', { path });
}

export function fileRename(oldPath: string, newPath: string): Promise<void> {
	return invoke('file_rename', { oldPath, newPath });
}

export function renameNote(
	systemPath: string,
	oldPath: string,
	newName: string,
): Promise<string> {
	return invoke('rename_note', { systemPath, oldPath, newName });
}

// Search
export function search(
	systemPath: string,
	query: string,
): Promise<SearchResult[]> {
	return invoke('search', { systemPath, query });
}

// Indexer
export function indexRebuild(systemPath: string): Promise<void> {
	return invoke('index_rebuild', { systemPath });
}

export function getBacklinks(
	systemPath: string,
	noteName: string,
): Promise<BacklinkInfo[]> {
	return invoke('get_backlinks_cmd', { systemPath, noteName });
}

export function getGraph(systemPath: string): Promise<GraphData> {
	return invoke('get_graph_cmd', { systemPath });
}

export function getTags(systemPath: string): Promise<TagEntry[]> {
	return invoke('get_tags_cmd', { systemPath });
}

export function findNote(
	systemPath: string,
	name: string,
): Promise<string | null> {
	return invoke('find_note', { systemPath, name });
}

// Semantic Search
export function semanticSearch(
	systemPath: string,
	query: string,
): Promise<SemanticResult[]> {
	return invoke('semantic_search_cmd', { systemPath, query });
}

export function semanticIndexRebuild(systemPath: string): Promise<void> {
	return invoke('semantic_index_rebuild_cmd', { systemPath });
}

export function testOllamaConnection(): Promise<OllamaHealth> {
	return invoke('test_ollama_connection_cmd');
}

export function checkDependencies(): Promise<DependencyStatus> {
	return invoke('check_dependencies_cmd');
}

// Settings
export function getSettings(): Promise<AppSettings> {
	return invoke('get_settings_cmd');
}

export function setSettings(settings: AppSettings): Promise<void> {
	return invoke('set_settings_cmd', { settings });
}

// PTY
export function ptyCreate(
	id: string,
	filePath: string,
	cols: number,
	rows: number,
	onData: Channel<Uint8Array>,
): Promise<void> {
	return invoke('pty_create', { id, filePath, cols, rows, onData });
}

export function ptyWrite(id: string, data: number[]): Promise<void> {
	return invoke('pty_write', { id, data });
}

export function ptyResize(
	id: string,
	cols: number,
	rows: number,
): Promise<void> {
	return invoke('pty_resize', { id, cols, rows });
}

export function ptyClose(id: string): Promise<void> {
	return invoke('pty_close', { id });
}

export function ptyCursorLine(id: string): Promise<number> {
	return invoke('pty_cursor_line', { id });
}

// Scratch Notes
export function scratchCreate(systemPath: string): Promise<string> {
	return invoke('scratch_create', { systemPath });
}

export function scratchList(systemPath: string): Promise<ScratchEntry[]> {
	return invoke('scratch_list', { systemPath });
}

export function generateNoteTitle(
	systemPath: string,
	path: string,
): Promise<string> {
	return invoke('generate_note_title', { systemPath, path });
}
