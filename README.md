# Synaptic

A minimal, neovim-driven desktop personal knowledge graph app.

Markdown files, live preview, embedded Neovim, graph view, semantic search.

## Shortcuts

| Shortcut | Action |
|----------|--------|
| `Cmd+P` | Find or create note |
| `Cmd+Shift+P` | Semantic search |
| `Cmd+G` | Graph view |
| `Cmd+N` | New note |

## macOS Gatekeeper

If Gatekeeper blocks the app:

```bash
sudo xattr -rd com.apple.quarantine /Applications/Synaptic.app
```
