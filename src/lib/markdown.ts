import { marked } from 'marked';
import katex from 'katex';
import hljs from 'highlight.js';

// Configure marked
marked.use({
  gfm: true,
  breaks: true,
});

// Wiki-link regex: [[Target]] or [[Target|Display]]
const WIKI_LINK_RE = /\[\[([^\]|]+)(?:\|([^\]]+))?\]\]/g;

// Math regex: $...$ inline, $$...$$ display
const INLINE_MATH_RE = /(?<!\$)\$(?!\$)([^$\n]+?)\$(?!\$)/g;
const DISPLAY_MATH_RE = /\$\$([\s\S]+?)\$\$/g;

// Frontmatter regex: ---\n...\n--- at the very start
const FRONTMATTER_RE = /^---\s*\n([\s\S]*?)\n---\s*(?:\n|$)/;

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function renderKatex(latex: string, displayMode: boolean): string {
  try {
    return katex.renderToString(latex, {
      displayMode,
      throwOnError: false,
      strict: false,
    });
  } catch {
    return `<span style="color:#cc4444">${escapeHtml(latex)}</span>`;
  }
}

function highlightCode(code: string, lang?: string): string {
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
export function parseFrontmatter(content: string): { tags: string[]; body: string } {
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
      value
        .slice(1, -1)
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)
        .forEach((t) => tags.add(t));
    } else {
      // Comma-separated string or single value
      value
        .split(',')
        .map((s) => s.trim())
        .filter(Boolean)
        .forEach((t) => tags.add(t));
    }
  }

  return { tags: Array.from(tags), body };
}

export function renderMarkdown(content: string): string {
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
  inlineMaths.forEach(({ placeholder, latex }) => {
    html = html.replace(placeholder, renderKatex(latex, false));
  });

  // 7. Restore display math
  displayMaths.forEach(({ placeholder, latex }) => {
    html = html.replace(placeholder, renderKatex(latex, true));
  });

  // 8. Syntax highlight code blocks
  // marked wraps code in <pre><code class="language-xxx">...</code></pre>
  html = html.replace(
    /<pre><code class="language-([^"]+)">([\s\S]*?)<\/code><\/pre>/g,
    (_, lang, code) => {
      // Decode HTML entities in code
      const decoded = code
        .replace(/&lt;/g, '<')
        .replace(/&gt;/g, '>')
        .replace(/&amp;/g, '&')
        .replace(/&quot;/g, '"');
      const highlighted = highlightCode(decoded, lang);
      return `<pre><code class="hljs language-${lang}">${highlighted}</code></pre>`;
    }
  );

  // Also handle code blocks without language
  html = html.replace(
    /<pre><code>([\s\S]*?)<\/code><\/pre>/g,
    (_, code) => {
      const decoded = code
        .replace(/&lt;/g, '<')
        .replace(/&gt;/g, '>')
        .replace(/&amp;/g, '&')
        .replace(/&quot;/g, '"');
      const highlighted = highlightCode(decoded, 'plaintext');
      return `<pre><code class="hljs">${highlighted}</code></pre>`;
    }
  );

  return html;
}
