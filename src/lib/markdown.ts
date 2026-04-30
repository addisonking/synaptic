import type { MarkedOptions } from 'marked';
import { marked } from 'marked';

// Configure marked
marked.use({
	gfm: true,
	breaks: true,
} as MarkedOptions);

// Wiki-link regex: [[Target]] or [[Target|Display]]
const WIKI_LINK_RE = /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g;

// Math regex: $...$ inline, $$...$$ display
const INLINE_MATH_RE = /(?<!\$)\$(?!\$)([^$\n]+?)\$(?!\$)/g;
const DISPLAY_MATH_RE = /\$\$([\s\S]+?)\$\$/g;

// Frontmatter regex: ---\n...\n--- at the very start
const FRONTMATTER_RE = /^---\s*\n([\s\S]*?)\n---\s*(?:\n|$)/;

// Lazy-loaded modules
let katexModule: typeof import('katex') | null = null;
let hljsModule: typeof import('highlight.js') | null = null;
let cssInjected = false;

async function injectMarkdownStyles() {
	if (cssInjected) return;
	cssInjected = true;
	await import('katex/dist/katex.min.css');
	await import('highlight.js/styles/github-dark.min.css');
}

async function getKatex() {
	if (!katexModule) {
		katexModule = (await import('katex')).default;
	}
	return katexModule;
}

async function getHljs() {
	if (!hljsModule) {
		hljsModule = (await import('highlight.js')).default;
	}
	return hljsModule;
}

function escapeHtml(text: string): string {
	return text
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}

async function renderKatex(
	latex: string,
	displayMode: boolean,
): Promise<string> {
	try {
		const katex = await getKatex();
		return katex.renderToString(latex, {
			displayMode,
			throwOnError: false,
			strict: false,
		});
	} catch {
		return `<span style="color:#cc4444">${escapeHtml(latex)}</span>`;
	}
}

async function highlightCode(code: string, lang?: string): Promise<string> {
	const hljs = await getHljs();
	if (lang && hljs.getLanguage(lang)) {
		try {
			return hljs.highlight(code, { language: lang }).value;
		} catch {
			return escapeHtml(code);
		}
	}
	return escapeHtml(code);
}

/**
 * Parse Obsidian-style YAML frontmatter.
 * Supports:
 *   tags: [a, b, c]
 *   tags: a, b, c
 *   tags:\n  - a\n  - b
 *   tag: single-value
 */
export function parseFrontmatter(content: string): {
	tags: string[];
	body: string;
} {
	const match = content.match(FRONTMATTER_RE);
	if (!match) return { tags: [], body: content };

	const raw = match[1];
	const body = content.slice(match[0].length);
	const tags = new Set<string>();

	// Look for tags/tag field in the YAML block
	const lines = raw.split('\n');
	let inTagsArray = false;
	let tagsIndent = 0;

	for (let i = 0; i < lines.length; i++) {
		const line = lines[i];
		const trim = line.trim();

		// End of a multi-line array when we hit a non-empty, non-indented line
		if (inTagsArray) {
			if (trim === '') continue;
			const indent = line.search(/\S/);
			if (indent <= tagsIndent && !trim.startsWith('-')) {
				inTagsArray = false;
				// Re-process this line if it might be another key
				i--;
				continue;
			}
			if (trim.startsWith('-')) {
				const val = trim.slice(1).trim();
				if (val) tags.add(val);
				continue;
			}
		}

		// Match tags: or tag: at start of line
		const m = line.match(/^(tags?|tags?):\s*(.*)$/);
		if (!m) continue;

		const value = m[2].trim();
		if (value === '') {
			// Multi-line array starting on next line
			inTagsArray = true;
			tagsIndent = line.search(/\S/);
		} else if (value.startsWith('[') && value.endsWith(']')) {
			// Inline array
			for (const t of value
				.slice(1, -1)
				.split(',')
				.map((s) => s.trim())
				.filter(Boolean)) {
				tags.add(t);
			}
		} else {
			// Comma-separated string or single value
			for (const t of value
				.split(',')
				.map((s) => s.trim())
				.filter(Boolean)) {
				tags.add(t);
			}
		}
	}

	return { tags: Array.from(tags), body };
}

export async function renderMarkdown(content: string): Promise<string> {
	await injectMarkdownStyles();

	// 1. Strip frontmatter before processing
	const { body } = parseFrontmatter(content);
	let text = body;

	// 2. Protect display math by replacing with placeholders
	const displayMaths: { placeholder: string; latex: string }[] = [];
	text = text.replace(DISPLAY_MATH_RE, (_, latex) => {
		const placeholder = `<!--DISPLAYMATH${displayMaths.length}-->`;
		displayMaths.push({ placeholder, latex });
		return placeholder;
	});

	// 3. Protect inline math
	const inlineMaths: { placeholder: string; latex: string }[] = [];
	text = text.replace(INLINE_MATH_RE, (_, latex) => {
		const placeholder = `<!--INLINEMATH${inlineMaths.length}-->`;
		inlineMaths.push({ placeholder, latex });
		return placeholder;
	});

	// 4. Convert wiki-links to markdown links before parsing
	text = text.replace(WIKI_LINK_RE, (_, target, display) => {
		const label = display ? display.trim() : target.trim();
		return `[${label}](wiki://${encodeURIComponent(target.trim())})`;
	});

	// 5. Parse markdown
	let html = marked.parse(text, { async: false }) as string;

	// 6. Restore inline math
	for (const { placeholder, latex } of inlineMaths) {
		html = html.replace(placeholder, await renderKatex(latex, false));
	}

	// 7. Restore display math
	for (const { placeholder, latex } of displayMaths) {
		html = html.replace(placeholder, await renderKatex(latex, true));
	}

	// 8. Syntax highlight code blocks
	// marked wraps code in <pre><code class="language-xxx">...</code></pre>
	// We can't use String.replace with async callbacks, so find matches then replace.
	const codeBlocks: { match: string; lang: string; code: string }[] = [];
	const codeRegex =
		/<pre><code class="language-([^"]+)">([\s\S]*?)<\/code><\/pre>/g;
	let cm: RegExpExecArray | null = codeRegex.exec(html);
	while (cm !== null) {
		codeBlocks.push({ match: cm[0], lang: cm[1], code: cm[2] });
		cm = codeRegex.exec(html);
	}

	const highlightedBlocks = await Promise.all(
		codeBlocks.map(async ({ lang, code }) => {
			if (lang === 'mermaid') {
				// keep entities encoded so the browser doesn't interpret < > as tags
				return `<div class="mermaid">${code}</div>`;
			}
			const decoded = code
				.replace(/&lt;/g, '<')
				.replace(/&gt;/g, '>')
				.replace(/&amp;/g, '&')
				.replace(/&quot;/g, '"');
			const highlighted = await highlightCode(decoded, lang);
			return `<pre><code class="hljs language-${lang}">${highlighted}</code></pre>`;
		}),
	);

	for (let i = 0; i < codeBlocks.length; i++) {
		html = html.replace(codeBlocks[i].match, () => highlightedBlocks[i]);
	}

	// Also handle code blocks without language
	const plainBlocks: { match: string; code: string }[] = [];
	const plainRegex = /<pre><code>([\s\S]*?)<\/code><\/pre>/g;
	let pm: RegExpExecArray | null = plainRegex.exec(html);
	while (pm !== null) {
		plainBlocks.push({ match: pm[0], code: pm[1] });
		pm = plainRegex.exec(html);
	}

	const highlightedPlain = await Promise.all(
		plainBlocks.map(async ({ code }) => {
			const decoded = code
				.replace(/&lt;/g, '<')
				.replace(/&gt;/g, '>')
				.replace(/&amp;/g, '&')
				.replace(/&quot;/g, '"');
			const highlighted = await highlightCode(decoded, 'plaintext');
			return `<pre><code class="hljs">${highlighted}</code></pre>`;
		}),
	);

	for (let i = 0; i < plainBlocks.length; i++) {
		html = html.replace(plainBlocks[i].match, () => highlightedPlain[i]);
	}

	return html;
}
