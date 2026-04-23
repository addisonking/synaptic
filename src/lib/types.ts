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
	ghost_model?: string;
}

export interface OllamaHealth {
	reachable: boolean;
	model_available: boolean;
	message: string;
}

export interface GhostSource {
	note_path: string;
	note_name: string;
	line: number;
	context: string;
	tags: string[];
}

export interface GhostLink {
	target: string;
	sources: GhostSource[];
}

export interface GhostNotePreview {
	target: string;
	content: string;
}
