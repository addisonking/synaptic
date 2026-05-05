<script lang="ts">
import { Search, X } from 'lucide-svelte';
import { onDestroy, onMount } from 'svelte';
import { fileRead, getGraph } from '$lib/api';
import { appState, openFile } from '$lib/store.svelte';

interface Props {
	onClose: () => void;
}

let { onClose }: Props = $props();

let canvas: HTMLCanvasElement;
let animFrame: number;
let running = true;

interface SimNode {
	id: string;
	path: string;
	link_count: number;
	kind: string;
	x: number;
	y: number;
	vx: number;
	vy: number;
	r: number;
}

interface SimEdge {
	source: string;
	target: string;
	a?: SimNode;
	b?: SimNode;
}

let nodes = $state<SimNode[]>([]);
let edges = $state<SimEdge[]>([]);
let nodeMap = new Map<string, SimNode>();
let connectedIds = new Set<string>();

let showOrphans = $state(true);
let showTags = $state(true);

// Search
let searchQuery = $state('');
let searchInputEl = $state<HTMLInputElement | null>(null);
let selectedNode: SimNode | null = null;

// Interaction
let dragging: SimNode | null = null;
let dragOffX = 0,
	dragOffY = 0;
let panX = 0,
	panY = 0;
let scale = 1;
let panning = false;
let panStartX = 0,
	panStartY = 0;
let hoveredNode: SimNode | null = null;
let alpha = 0.2; // simulation heat — start low for gentle entry

const currentName = $derived(
	appState.openFilePath?.split('/').pop()?.replace(/\.md$/, '') ?? '',
);

onMount(async () => {
	if (!appState.system) return;
	resize();

	const data = await getGraph(appState.system.path);

	const cx = canvas.width / 2;
	const cy = canvas.height / 2;

	// Place nodes in a loose circle to avoid initial clustering explosions
	const count = data.nodes.length;
	const radius = Math.max(200, count * 8);
	nodes = data.nodes.map((n, i) => {
		const angle = (i / count) * Math.PI * 2 + (Math.random() - 0.5) * 0.3;
		const r = radius * (0.7 + Math.random() * 0.3);
		const kind = n.kind ?? 'note';
		return {
			...n,
			kind,
			x: cx + Math.cos(angle) * r,
			y: cy + Math.sin(angle) * r,
			vx: 0,
			vy: 0,
			r: kind === 'tag' ? tagRadius(n.link_count) : nodeRadius(n.link_count),
		};
	});
	nodeMap = new Map(nodes.map((n) => [n.id, n]));
	edges = data.edges.map((e) => ({
		...e,
		a: nodeMap.get(e.source),
		b: nodeMap.get(e.target),
	}));
	connectedIds = new Set(edges.flatMap((e) => [e.source, e.target]));

	// Center view on current note
	const cur = nodeMap.get(currentName);
	if (cur) {
		panX = canvas.width / 2 - cur.x;
		panY = canvas.height / 2 - cur.y;
	}

	loop();

	const ro = new ResizeObserver(resize);
	ro.observe(canvas);
	return () => ro.disconnect();
});

onDestroy(() => {
	running = false;
	cancelAnimationFrame(animFrame);
});

function nodeRadius(linkCount: number) {
	return 4 + Math.sqrt(linkCount) * 2.5;
}

function tagRadius(noteCount: number) {
	return 3 + Math.sqrt(noteCount) * 1.8;
}

function isTagEdge(e: SimEdge) {
	return e.source.startsWith('#') || e.target.startsWith('#');
}

// Continuous simulation with cooling
function tick() {
	if (alpha < 0.001) return;
	alpha *= 0.992;

	const REPEL = 600;
	const LINK_DIST = 140;
	const ATTRACT = 0.04;
	const CENTER_PULL = 0.003;
	const DAMP = 0.82;
	const cx = canvas ? canvas.width / 2 : 0;
	const cy = canvas ? canvas.height / 2 : 0;

	// Repulsion
	for (let i = 0; i < nodes.length; i++) {
		for (let j = i + 1; j < nodes.length; j++) {
			const a = nodes[i],
				b = nodes[j];
			const dx = b.x - a.x;
			const dy = b.y - a.y;
			const dist2 = Math.max(dx * dx + dy * dy, 400); // clamp to prevent infinity forces
			const dist = Math.sqrt(dist2);
			const minDist = a.r + b.r + 20;
			if (dist < minDist * 4) {
				const force = (REPEL / dist2) * alpha;
				const nx = dx / dist,
					ny = dy / dist;
				a.vx -= nx * force;
				a.vy -= ny * force;
				b.vx += nx * force;
				b.vy += ny * force;
			}
		}
	}

	// Link spring
	for (const e of edges) {
		const a = e.a,
			b = e.b;
		if (!a || !b) continue;
		const dx = b.x - a.x,
			dy = b.y - a.y;
		const dist = Math.sqrt(dx * dx + dy * dy) || 1;
		const displacement = dist - LINK_DIST;
		const force = displacement * ATTRACT * alpha;
		const nx = dx / dist,
			ny = dy / dist;
		a.vx += nx * force;
		a.vy += ny * force;
		b.vx -= nx * force;
		b.vy -= ny * force;
	}

	// Center gravity
	for (const n of nodes) {
		n.vx += (cx - n.x) * CENTER_PULL * alpha;
		n.vy += (cy - n.y) * CENTER_PULL * alpha;
	}

	// Integrate
	for (const n of nodes) {
		if (n === dragging) continue;
		n.vx *= DAMP;
		n.vy *= DAMP;
		n.x += n.vx;
		n.y += n.vy;
	}
}

function draw() {
	if (!canvas) return;
	const ctx = canvas.getContext('2d');
	if (!ctx) return;
	const w = canvas.width,
		h = canvas.height;

	ctx.clearRect(0, 0, w, h);
	ctx.save();
	ctx.translate(panX, panY);
	ctx.scale(scale, scale);

	// Determine highlighted sets
	const neighborIds = new Set<string>();
	if (hoveredNode) {
		neighborIds.add(hoveredNode.id);
		for (const e of edges) {
			if (e.source === hoveredNode.id) neighborIds.add(e.target);
			if (e.target === hoveredNode.id) neighborIds.add(e.source);
		}
	}
	const selectedNeighborIds = new Set<string>();
	if (selectedNode) {
		selectedNeighborIds.add(selectedNode.id);
		for (const e of edges) {
			if (e.source === selectedNode.id) selectedNeighborIds.add(e.target);
			if (e.target === selectedNode.id) selectedNeighborIds.add(e.source);
		}
	}
	const hasHover = hoveredNode !== null;
	const hasSelection = selectedNode !== null;

	// Edges
	for (const e of edges) {
		const a = e.a,
			b = e.b;
		if (!a || !b) continue;
		const tagEdge = isTagEdge(e);
		if (tagEdge && !showTags) continue;

		const isNeighborEdge =
			hasHover &&
			(e.source === hoveredNode?.id || e.target === hoveredNode?.id);
		const isSelectedEdge =
			hasSelection &&
			(e.source === selectedNode?.id || e.target === selectedNode?.id);
		const isCurEdge = e.source === currentName || e.target === currentName;

		ctx.beginPath();
		ctx.moveTo(a.x, a.y);
		ctx.lineTo(b.x, b.y);

		if (tagEdge) {
			ctx.setLineDash([3, 4]);
			if (isNeighborEdge || isSelectedEdge) {
				ctx.strokeStyle = '#2a6060';
				ctx.lineWidth = 1;
			} else {
				ctx.strokeStyle = '#182828';
				ctx.lineWidth = 0.8;
			}
		} else {
			ctx.setLineDash([]);
			if (hasSelection) {
				if (isSelectedEdge) {
					ctx.strokeStyle = '#555544';
					ctx.lineWidth = 2;
				} else if (hasHover && isNeighborEdge) {
					ctx.strokeStyle = '#444444';
					ctx.lineWidth = 1.5;
				} else {
					ctx.strokeStyle = '#0f0f0f';
					ctx.lineWidth = 0.5;
				}
			} else if (hasHover) {
				ctx.strokeStyle = isNeighborEdge ? '#444444' : '#0f0f0f';
				ctx.lineWidth = isNeighborEdge ? 1.5 : 0.5;
			} else {
				ctx.strokeStyle = isCurEdge ? '#333333' : '#1c1c1c';
				ctx.lineWidth = isCurEdge ? 1.5 : 1;
			}
		}
		ctx.stroke();
		ctx.setLineDash([]);
	}

	// Nodes
	for (const n of nodes) {
		const isTag = n.kind === 'tag';
		if (isTag && !showTags) continue;
		if (!showOrphans && !connectedIds.has(n.id) && !isTag) continue;

		const isCurrent = n.id === currentName;
		const isHovered = n === hoveredNode;
		const isNeighbor = neighborIds.has(n.id);
		const isSelected = n === selectedNode;
		const isSelectedNeighbor = selectedNeighborIds.has(n.id);
		const r = n.r / scale;

		let fillColor: string;
		if (isTag) {
			if (isSelected) fillColor = '#3a9a9a';
			else if (isHovered) fillColor = '#2a7070';
			else if (isNeighbor && hasHover) fillColor = '#224444';
			else if (isSelectedNeighbor) fillColor = '#1e4040';
			else fillColor = '#162e2e';
		} else {
			if (isSelected) fillColor = '#ccaa44';
			else if (isCurrent) fillColor = '#ffffff';
			else if (isHovered) fillColor = '#cccccc';
			else if (hasHover && isNeighbor) fillColor = '#888888';
			else if (hasHover) fillColor = '#1a1a1a';
			else if (hasSelection && isSelectedNeighbor) fillColor = '#666666';
			else if (hasSelection && !isSelectedNeighbor) fillColor = '#1a1a1a';
			else if (n.link_count > 5) fillColor = '#666666';
			else if (n.link_count > 0) fillColor = '#444444';
			else fillColor = '#2a2a2a';
		}

		if (isTag) {
			// Diamond shape
			ctx.beginPath();
			ctx.moveTo(n.x, n.y - r * 1.3);
			ctx.lineTo(n.x + r, n.y);
			ctx.lineTo(n.x, n.y + r * 1.3);
			ctx.lineTo(n.x - r, n.y);
			ctx.closePath();
			ctx.fillStyle = fillColor;
			ctx.fill();
			if (isSelected || isHovered) {
				ctx.strokeStyle = isSelected ? '#3a9a9a' : '#2a6060';
				ctx.lineWidth = 1;
				ctx.stroke();
			}
		} else {
			ctx.beginPath();
			ctx.arc(n.x, n.y, r, 0, Math.PI * 2);
			ctx.fillStyle = fillColor;
			ctx.fill();
			if (isSelected) {
				ctx.beginPath();
				ctx.arc(n.x, n.y, r + 3, 0, Math.PI * 2);
				ctx.strokeStyle = '#ccaa44';
				ctx.lineWidth = 1.5;
				ctx.stroke();
			}
		}
	}

	// Labels
	const showAllLabels = scale > 0.8;
	ctx.textBaseline = 'middle';

	for (const n of nodes) {
		const isTag = n.kind === 'tag';
		if (isTag && !showTags) continue;
		if (!showOrphans && !connectedIds.has(n.id) && !isTag) continue;

		const isCurrent = n.id === currentName;
		const isHovered = n === hoveredNode;
		const isNeighbor = neighborIds.has(n.id);
		const isSelected = n === selectedNode;
		const isSelectedNeighbor = selectedNeighborIds.has(n.id);

		const shouldShow =
			isCurrent ||
			isHovered ||
			isNeighbor ||
			isSelected ||
			isSelectedNeighbor ||
			showAllLabels;
		if (!shouldShow) continue;

		const dimmed =
			(hasHover && !isNeighbor && !isCurrent && !isSelected) ||
			(hasSelection && !isSelectedNeighbor && !isSelected && !isCurrent);

		const fontSize = Math.max(9, 11 / scale);
		ctx.font = `${isCurrent || isSelected ? 600 : 400} ${fontSize}px "Geist Mono", monospace`;

		if (isTag) {
			if (dimmed) ctx.fillStyle = '#1a3030';
			else if (isSelected || isHovered) ctx.fillStyle = '#3a9a9a';
			else if (isNeighbor || isSelectedNeighbor) ctx.fillStyle = '#2a6060';
			else ctx.fillStyle = '#254545';
		} else {
			if (dimmed) ctx.fillStyle = '#222222';
			else if (isSelected) ctx.fillStyle = '#ccaa44';
			else if (isCurrent || isHovered) ctx.fillStyle = '#ffffff';
			else if (isNeighbor || isSelectedNeighbor) ctx.fillStyle = '#888888';
			else ctx.fillStyle = '#555555';
		}

		const r = n.r / scale;
		ctx.fillText(n.id, n.x + r + 4 / scale, n.y);
	}

	ctx.restore();
}

function ensureLoop() {
	if (running && animFrame === 0) {
		loop();
	}
}

function loop() {
	if (!running) return;
	tick();
	draw();
	if (alpha >= 0.001 || dragging || panning) {
		animFrame = requestAnimationFrame(loop);
	} else {
		animFrame = 0;
	}
}

// --- Input ---

function toWorld(clientX: number, clientY: number) {
	const rect = canvas.getBoundingClientRect();
	return {
		x: (clientX - rect.left - panX) / scale,
		y: (clientY - rect.top - panY) / scale,
	};
}

function hitTest(clientX: number, clientY: number): SimNode | null {
	const { x, y } = toWorld(clientX, clientY);
	for (let i = nodes.length - 1; i >= 0; i--) {
		const n = nodes[i];
		if (n.kind === 'tag' && !showTags) continue;
		if (!showOrphans && !connectedIds.has(n.id) && n.kind !== 'tag') continue;
		const r = n.r / scale + 4 / scale;
		if ((n.x - x) ** 2 + (n.y - y) ** 2 <= r * r) return n;
	}
	return null;
}

let clickNode: SimNode | null = null;
let mouseDownTime = 0;

function onMouseDown(e: MouseEvent) {
	if (e.button !== 0) return;
	mouseDownTime = Date.now();
	const hit = hitTest(e.clientX, e.clientY);
	if (hit) {
		dragging = hit;
		clickNode = hit;
		const { x, y } = toWorld(e.clientX, e.clientY);
		dragOffX = x - hit.x;
		dragOffY = y - hit.y;
		alpha = Math.max(alpha, 0.3);
	} else {
		panning = true;
		panStartX = e.clientX - panX;
		panStartY = e.clientY - panY;
	}
	ensureLoop();
}

function onMouseMove(e: MouseEvent) {
	if (dragging) {
		const { x, y } = toWorld(e.clientX, e.clientY);
		dragging.x = x - dragOffX;
		dragging.y = y - dragOffY;
		dragging.vx = 0;
		dragging.vy = 0;
		clickNode = null; // moved — not a click
	} else if (panning) {
		panX = e.clientX - panStartX;
		panY = e.clientY - panStartY;
	} else {
		const prev = hoveredNode;
		hoveredNode = hitTest(e.clientX, e.clientY);
		canvas.style.cursor = hoveredNode ? 'pointer' : 'default';
		if (hoveredNode !== prev) {
			alpha = Math.max(alpha, 0.05);
			ensureLoop();
		}
	}
}

async function onMouseUp(_e: MouseEvent) {
	const elapsed = Date.now() - mouseDownTime;
	if (clickNode && elapsed < 300) {
		if (clickNode.path) {
			const content = await fileRead(clickNode.path);
			openFile(clickNode.path, content);
			onClose();
		}
	}
	dragging = null;
	clickNode = null;
	panning = false;
}

function onWheel(e: WheelEvent) {
	e.preventDefault();
	const rect = canvas.getBoundingClientRect();
	const mx = e.clientX - rect.left;
	const my = e.clientY - rect.top;
	const factor = Math.max(0.85, Math.min(1.18, 0.998 ** e.deltaY));
	panX = mx - (mx - panX) * factor;
	panY = my - (my - panY) * factor;
	scale = Math.max(0.15, Math.min(6, scale * factor));
	ensureLoop();
}

function resetView() {
	scale = 1;
	const cur = nodeMap.get(currentName);
	if (cur) {
		panX = canvas.width / 2 - cur.x;
		panY = canvas.height / 2 - cur.y;
	} else {
		panX = 0;
		panY = 0;
	}
	alpha = 0.15;
	ensureLoop();
}

function centerOnNode(n: SimNode) {
	if (!canvas) return;
	selectedNode = n;
	const targetPanX = canvas.width / 2 - n.x * scale;
	const targetPanY = canvas.height / 2 - n.y * scale;

	const startPanX = panX;
	const startPanY = panY;
	const duration = 400;
	const startTime = performance.now();

	function animate(now: number) {
		const t = Math.min((now - startTime) / duration, 1);
		const ease = 1 - (1 - t) ** 3;
		panX = startPanX + (targetPanX - startPanX) * ease;
		panY = startPanY + (targetPanY - startPanY) * ease;
		draw();
		if (t < 1) requestAnimationFrame(animate);
	}
	requestAnimationFrame(animate);
}

function doSearch() {
	const q = searchQuery.trim().toLowerCase();
	if (q.length < 1) {
		selectedNode = null;
		return;
	}
	const matches = nodes.filter((n) => {
		if (!showOrphans && !connectedIds.has(n.id)) return false;
		return n.id.toLowerCase().includes(q);
	});
	if (matches.length > 0) {
		// Score: exact (3) > starts-with (2) > contains (1), then rank by link_count
		const scored = matches.map((n) => {
			const name = n.id.toLowerCase();
			let score = 0;
			if (name === q) score = 3;
			else if (name.startsWith(q)) score = 2;
			else score = 1;
			return { node: n, score, links: n.link_count };
		});
		scored.sort((a, b) => {
			if (b.score !== a.score) return b.score - a.score;
			return b.links - a.links;
		});
		const best = scored[0].node;
		selectedNode = best;
		centerOnNode(best);
		ensureLoop();
	} else {
		selectedNode = null;
		ensureLoop();
	}
}

function clearSearch() {
	searchQuery = '';
	selectedNode = null;
	ensureLoop();
}

function resize() {
	if (!canvas) return;
	canvas.width = canvas.offsetWidth;
	canvas.height = canvas.offsetHeight;
}

async function openSelectedNode() {
	if (!selectedNode) return;
	if (selectedNode.path) {
		const content = await fileRead(selectedNode.path);
		openFile(selectedNode.path, content);
		onClose();
	}
}

function onKeydown(e: KeyboardEvent) {
	if (e.key === 'Escape') {
		if (document.activeElement === searchInputEl || selectedNode) {
			clearSearch();
			searchInputEl?.blur();
			return;
		}
		onClose();
	}
	if (e.key === '/') {
		e.preventDefault();
		searchInputEl?.focus();
	}
	if (e.key === 'Enter') {
		if (document.activeElement === searchInputEl) {
			e.preventDefault();
			searchInputEl?.blur();
		}
		openSelectedNode();
	}
}
</script>

<svelte:window onkeydown={onKeydown} />

<div class="graph-overlay" onclick={(e) => e.target === e.currentTarget && onClose()}>
  <div class="graph-panel">
    <div class="graph-header">
      <span class="graph-title">graph</span>
      <span class="graph-meta">
        {nodes.filter(n => n.kind !== 'tag').length} notes
        · {nodes.filter(n => n.kind === 'tag').length} tags
        · {edges.filter(e => !isTagEdge(e)).length} links
      </span>

      <button
        class="tag-toggle"
        class:active={showTags}
        onclick={() => { showTags = !showTags; ensureLoop(); }}
        title="Toggle tag nodes"
      >tags</button>

      <div class="graph-search-wrap">
        <div class="graph-search">
          <Search size={12} />
          <input
            bind:this={searchInputEl}
            bind:value={searchQuery}
            oninput={doSearch}
            placeholder="Search nodes (/ to focus)"
            type="text"
          />
          {#if searchQuery}
            <button class="search-clear" onclick={clearSearch}><X size={12} /></button>
          {/if}
        </div>
      </div>

      <div class="graph-actions">
        <button class="close-btn" onclick={onClose} title="Close">
          <X size={14} />
        </button>
      </div>
    </div>
    <canvas
      bind:this={canvas}
      class="graph-canvas"
      onmousedown={onMouseDown}
      onmousemove={onMouseMove}
      onmouseup={onMouseUp}
      onwheel={onWheel}
    ></canvas>
    <div class="graph-hint">scroll to zoom · drag to pan · click node to open · / to search</div>
  </div>
</div>

<style>
  .graph-overlay {
    position: fixed;
    inset: 0;
    background: var(--bg);
    z-index: 90;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }

  .graph-panel {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .graph-header {
    height: 40px;
    border-bottom: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
    flex-shrink: 0;
  }

  .graph-title {
    color: var(--text-bright);
    font-size: 13px;
    font-weight: 500;
  }

  .graph-meta {
    color: var(--muted-2);
    font-size: 12px;
    margin-left: 12px;
    flex: 1;
  }

  .graph-actions {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .action-btn {
    padding: 4px 10px;
    font-size: 12px;
    color: var(--text);
    border: 1px solid transparent;
    transition: background 80ms, border-color 80ms;
    background: transparent;
    cursor: pointer;
  }

  .action-btn:hover {
    background: var(--surface-1);
    border-color: var(--surface-2);
  }

  .action-btn.active {
    background: var(--surface-2);
    border-color: var(--surface-3);
    color: var(--text-bright);
  }

  .icon-btn {
    padding: 4px 8px;
    font-size: 13px;
  }

  .tag-toggle {
    padding: 3px 10px;
    font-size: 11px;
    color: var(--muted-3);
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    transition: color 80ms, border-color 80ms, background 80ms;
    margin-right: 8px;
  }

  .tag-toggle:hover {
    color: #3a9a9a;
    border-color: #1a3a3a;
  }

  .tag-toggle.active {
    color: #3a9a9a;
    border-color: #1e4040;
    background: #0e1e1e;
  }

  .close-btn {
    padding: 4px 8px;
    font-size: 13px;
    color: var(--muted-3);
    border: 1px solid transparent;
    background: transparent;
    cursor: pointer;
    transition: color 80ms;
  }

  .close-btn:hover {
    color: var(--text-bright);
  }

  .graph-canvas {
    flex: 1;
    width: 100%;
    display: block;
    cursor: default;
  }

  .graph-hint {
    height: 28px;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--muted-2);
    font-size: 11px;
    flex-shrink: 0;
  }

  .graph-search-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .graph-search {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--bg);
    border: 1px solid var(--surface-2);
    padding: 4px 8px;
    transition: border-color 80ms;
  }

  .graph-search input {
    background: transparent;
    border: none;
    padding: 0;
    font-size: 12px;
    color: var(--text-bright);
    width: 160px;
    outline: none;
  }

  .graph-search input::placeholder {
    color: var(--muted-2);
  }

  .search-clear {
    display: flex;
    align-items: center;
    color: var(--muted-2);
    transition: color 80ms;
    padding: 0;
  }

  .search-clear:hover {
    color: var(--text-bright);
  }
</style>
