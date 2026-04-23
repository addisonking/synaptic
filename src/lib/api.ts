import { type Channel, invoke } from '@tauri-apps/api/core';
import type {
	AppSettings,
	BacklinkInfo,
	FileNode,
	GhostLink,
	GhostNotePreview,
	GhostSource,
	GraphData,
	OllamaHealth,
	SearchResult,
	SemanticResult,
	SystemInfo,
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

// Settings
export function getSettings(): Promise<AppSettings> {
	return invoke('get_settings_cmd');
}

export function setSettings(settings: AppSettings): Promise<void> {
	return invoke('set_settings_cmd', { settings });
}

// Ghost Links
export function scanGhostLinks(systemPath: string): Promise<GhostLink[]> {
	return invoke('scan_ghost_links_cmd', { systemPath });
}

export function previewGhostNote(
	systemPath: string,
	target: string,
	sources: GhostSource[],
): Promise<GhostNotePreview> {
	return invoke('preview_ghost_note_cmd', { systemPath, target, sources });
}

export function previewGhostNoteStream(
	systemPath: string,
	target: string,
	sources: GhostSource[],
	onChunk: Channel<{ kind: string; data: string }>,
): Promise<void> {
	return invoke('preview_ghost_note_stream_cmd', {
		systemPath,
		target,
		sources,
		onChunk,
	});
}

export function createGhostNotes(
	systemPath: string,
	notes: GhostNotePreview[],
): Promise<void> {
	return invoke('create_ghost_notes_cmd', { systemPath, notes });
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
