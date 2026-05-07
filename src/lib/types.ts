export interface SystemInfo {
	name: string;
	uuid: string;
	path: string;
	created: number;
}

export interface FileNode {
	name: string;
	path: string;
	isDirectory: boolean;
	children?: FileNode[];
}

export interface SearchResult {
	path: string;
	name: string;
	line: number;
	content: string;
}

export interface BacklinkInfo {
	note_name: string;
	note_path: string;
}

export interface GraphNode {
	id: string;
	path: string;
	link_count: number;
	kind?: string;
}

export interface TagNote {
	name: string;
	path: string;
}

export interface TagEntry {
	tag: string;
	count: number;
	notes: TagNote[];
}

export interface GraphEdge {
	source: string;
	target: string;
}

export interface GraphData {
	nodes: GraphNode[];
	edges: GraphEdge[];
}

export interface SemanticResult {
	path: string;
	name: string;
	line: number;
	content: string;
	score: number;
}

export interface AppSettings {
	nvim_path?: string;
	ollama_url?: string;
	ollama_model?: string;
	generation_model?: string;
	github_sync_enabled?: boolean;
	github_repo_url?: string;
	github_token?: string;
	github_branch?: string;
}

export interface SyncState {
	syncing: boolean;
	last_error: string | null;
	last_sync: number | null;
}

export interface GitCheckResult {
	installed: boolean;
}

export interface RepoValidationResult {
	valid: boolean;
	message: string;
}

export interface OllamaHealth {
	reachable: boolean;
	model_available: boolean;
	message: string;
}

export interface DependencyStatus {
	neovim_installed: boolean;
	neovim_path: string | null;
	ollama_running: boolean;
	ollama_url: string;
	platform: string;
	message: string;
}

export interface ScratchEntry {
	name: string;
	path: string;
	modified: number;
}
