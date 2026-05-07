<script lang="ts">
import { Check, X } from 'lucide-svelte';
import {
	checkGitInstalled,
	getSettings,
	semanticIndexRebuild,
	setSettings,
	syncNow,
	testOllamaConnection,
	validateGitHubToken,
} from '$lib/api';
import { Button, Dialog, Input, Tabs } from '$lib/components/ui';
import { appState } from '$lib/store.svelte';
import type {
	AppSettings,
	GitCheckResult,
	OllamaHealth,
	RepoValidationResult,
} from '$lib/types';
import { version as appVersion } from '../../../package.json';

interface Props {
	onClose: () => void;
}

let { onClose }: Props = $props();
let open = $state(true);
let activeTab = $state('general');
let settings = $state<AppSettings>({});
let saveStatus = $state<'saved' | 'saving' | 'error'>('saved');
let rebuildStatus = $state<'idle' | 'building' | 'done' | 'error'>('idle');
let rebuildError = $state<string | null>(null);
let testStatus = $state<'idle' | 'testing' | OllamaHealth['message']>('idle');
let testOk = $state<boolean | null>(null);
let gitCheck = $state<GitCheckResult | null>(null);
let tokenTestStatus = $state<
	'idle' | 'testing' | RepoValidationResult['message']
>('idle');
let tokenTestOk = $state<boolean | null>(null);
let syncNowStatus = $state<'idle' | 'syncing' | 'done' | 'error'>('idle');
let syncNowError = $state<string | null>(null);

$effect(() => {
	if (!open) {
		onClose();
	}
});

$effect(() => {
	getSettings().then((s) => {
		settings = { ...s };
	});
});

async function saveSettings() {
	saveStatus = 'saving';
	try {
		await setSettings(settings);
		saveStatus = 'saved';
	} catch {
		saveStatus = 'error';
	}
}

async function handleRebuild() {
	if (!appState.system) return;
	rebuildStatus = 'building';
	rebuildError = null;
	try {
		await semanticIndexRebuild(appState.system.path);
		rebuildStatus = 'done';
	} catch (e) {
		rebuildStatus = 'error';
		rebuildError = String(e);
	}
}

async function handleTestConnection() {
	testStatus = 'testing';
	testOk = null;
	try {
		const health = await testOllamaConnection();
		testStatus = health.message;
		testOk = health.reachable && health.model_available;
	} catch (e) {
		testStatus = String(e);
		testOk = false;
	}
}

async function handleCheckGit() {
	gitCheck = await checkGitInstalled();
}

async function handleTestToken() {
	if (!settings.github_token || !settings.github_repo_url) return;
	tokenTestStatus = 'testing';
	tokenTestOk = null;
	try {
		const result = await validateGitHubToken(
			settings.github_token,
			settings.github_repo_url,
		);
		tokenTestStatus = result.message;
		tokenTestOk = result.valid;
	} catch (e) {
		tokenTestStatus = String(e);
		tokenTestOk = false;
	}
}

async function handleSyncNow() {
	if (!appState.system) return;
	syncNowStatus = 'syncing';
	syncNowError = null;
	try {
		await syncNow(appState.system.path);
		syncNowStatus = 'done';
	} catch (e) {
		syncNowStatus = 'error';
		syncNowError = String(e);
	}
}

const tabItems = [
	{ value: 'general', label: 'General' },
	{ value: 'ai', label: 'AI / Search' },
	{ value: 'sync', label: 'Sync' },
];
</script>

<Dialog bind:open title="Settings" contentClass="settings-dialog">
  <div class="content">
    <Tabs bind:value={activeTab} items={tabItems}>
      <div class="tab-panel">
        {#if activeTab === 'general'}
          <div class="field">
            <label for="nvim-path">Neovim Path</label>
            <Input
              id="nvim-path"
              type="text"
              placeholder="/usr/local/bin/nvim"
              bind:value={settings.nvim_path}
              onchange={saveSettings}
            />
            <span class="hint">Path to the nvim executable used in the terminal pane.</span>
          </div>


        {:else if activeTab === 'ai'}
          <div class="field">
            <label for="ollama-url">Ollama URL</label>
            <Input
              id="ollama-url"
              type="text"
              placeholder="http://localhost:11434"
              bind:value={settings.ollama_url}
              onchange={saveSettings}
            />
            <span class="hint">Base URL of your Ollama instance.</span>
          </div>

          <div class="field">
            <label for="ollama-model">Embedding Model</label>
            <Input
              id="ollama-model"
              type="text"
              placeholder="nomic-embed-text"
              bind:value={settings.ollama_model}
              onchange={saveSettings}
            />
            <span class="hint">Model used for semantic search embeddings.</span>
          </div>

          <div class="field">
            <label for="generation-model">Generation Model</label>
            <Input
              id="generation-model"
              type="text"
              placeholder="gemma4:26b"
              bind:value={settings.generation_model}
              onchange={saveSettings}
            />
            <span class="hint">Model used for AI-generated content (e.g. scratch note titles).</span>
          </div>

          <div class="field">
            <span class="field-label">Connection</span>
            <Button
              variant="default"
              size="sm"
              onclick={handleTestConnection}
              disabled={testStatus === 'testing'}
            >
              {#if testStatus === 'testing'}
                Testing…
              {:else if testStatus === 'idle'}
                Test Connection
              {:else}
                {#if testOk === true}<Check size={14} />{:else}<X size={14} />{/if} {testStatus}
              {/if}
            </Button>
            <span class="hint">Verify that Ollama is running and the model is available.</span>
          </div>

          <div class="field">
            <span class="field-label">Embeddings Index</span>
            <Button
              variant="default"
              size="sm"
              onclick={handleRebuild}
              disabled={rebuildStatus === 'building'}
            >
              {#if rebuildStatus === 'building'}
                Indexing…
              {:else if rebuildStatus === 'done'}
                <Check size={14} /> Index rebuilt
              {:else if rebuildStatus === 'error'}
                <X size={14} /> Rebuild failed
              {:else}
                Rebuild Index
              {/if}
            </Button>
            {#if rebuildError}
              <span class="hint error">{rebuildError}</span>
            {:else}
              <span class="hint">Regenerate embeddings for all notes. This may take a while.</span>
            {/if}
          </div>
        {:else if activeTab === 'sync'}
          <div class="field">
            <span class="field-label">Git</span>
            {#if gitCheck}
              <span class="hint">
                {#if gitCheck.installed}
                  <Check size={14} /> Git is installed
                {:else}
                  <X size={14} /> Git not found. Install it to enable sync.
                {/if}
              </span>
            {:else}
              <Button variant="default" size="sm" onclick={handleCheckGit}>Check Git</Button>
            {/if}
          </div>

          <div class="field">
            <label for="sync-enabled">Auto Sync</label>
            <input
              id="sync-enabled"
              type="checkbox"
              checked={settings.github_sync_enabled ?? false}
              onchange={(e) => {
                settings.github_sync_enabled = (e.target as HTMLInputElement).checked;
                saveSettings();
              }}
            />
            <span class="hint">Automatically push changes to GitHub every 5 minutes.</span>
          </div>

          <div class="field">
            <label for="repo-url">Repository URL</label>
            <Input
              id="repo-url"
              type="text"
              placeholder="https://github.com/username/repo"
              value={settings.github_repo_url ?? ''}
              onchange={(e) => {
                settings.github_repo_url = (e.target as HTMLInputElement).value;
                saveSettings();
              }}
            />
            <span class="hint">The GitHub repo to push to. The vault will be auto-initialized as a git repo.</span>
          </div>

          <div class="field">
            <label for="github-token">Personal Access Token</label>
            <Input
              id="github-token"
              type="password"
              placeholder="ghp_..."
              value={settings.github_token ?? ''}
              onchange={(e) => {
                settings.github_token = (e.target as HTMLInputElement).value;
                saveSettings();
              }}
            />
            <span class="hint">GitHub PAT with repo scope. Stored locally in settings.</span>
          </div>

          <div class="field">
            <label for="github-branch">Branch</label>
            <Input
              id="github-branch"
              type="text"
              placeholder="main"
              value={settings.github_branch ?? ''}
              onchange={(e) => {
                settings.github_branch = (e.target as HTMLInputElement).value;
                saveSettings();
              }}
            />
            <span class="hint">Branch to push to. Defaults to main.</span>
          </div>

          <div class="field">
            <span class="field-label">Connection</span>
            <Button
              variant="default"
              size="sm"
              onclick={handleTestToken}
              disabled={tokenTestStatus === 'testing' || !settings.github_token || !settings.github_repo_url}
            >
              {#if tokenTestStatus === 'testing'}
                Testing…
              {:else if tokenTestStatus === 'idle'}
                Test Connection
              {:else}
                {#if tokenTestOk === true}<Check size={14} />{:else}<X size={14} />{/if} {tokenTestStatus}
              {/if}
            </Button>
            <span class="hint">Verify that your token can access the repository.</span>
          </div>

          <div class="field">
            <span class="field-label">Manual Sync</span>
            <Button
              variant="default"
              size="sm"
              onclick={handleSyncNow}
              disabled={syncNowStatus === 'syncing' || !appState.system}
            >
              {#if syncNowStatus === 'syncing'}
                Syncing…
              {:else if syncNowStatus === 'done'}
                <Check size={14} /> Synced
              {:else if syncNowStatus === 'error'}
                <X size={14} /> Sync failed
              {:else}
                Sync Now
              {/if}
            </Button>
            {#if syncNowError}
              <span class="hint error">{syncNowError}</span>
            {:else}
              <span class="hint">Push changes to GitHub immediately.</span>
            {/if}
          </div>

          {#if appState.syncStatus.last_sync}
            <div class="field">
              <span class="hint">Last sync: {new Date(appState.syncStatus.last_sync * 1000).toLocaleString()}</span>
            </div>
          {/if}
          {#if appState.syncStatus.last_error}
            <div class="field">
              <span class="hint error">Last error: {appState.syncStatus.last_error}</span>
            </div>
          {/if}
        {/if}
      </div>
    </Tabs>

    <div class="footer">
      <span class="version">v{appVersion}</span>
      <span class="save-indicator" class:error={saveStatus === 'error'}>
        {#if saveStatus === 'saving'}
          Saving…
        {:else if saveStatus === 'error'}
          Save failed
        {/if}
      </span>
      <Button variant="default" size="sm" onclick={() => (open = false)}>Close</Button>
    </div>
  </div>
</Dialog>

<style>
  .content {
    width: 520px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .tab-panel {
    padding: 20px;
    flex: 1;
    overflow-y: auto;
  }
  .field {
    margin-bottom: 20px;
  }
  .field:last-child {
    margin-bottom: 0;
  }
  .field label,
  .field-label {
    display: block;
    font-size: 12px;
    color: var(--text-bright);
    margin-bottom: 6px;
  }
  .hint {
    display: block;
    font-size: 11px;
    color: var(--muted-2);
    margin-top: 6px;
  }
  .hint.error {
    color: var(--error);
  }
  .footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 20px;
    border-top: 1px solid var(--border);
  }
  .save-indicator {
    font-size: 11px;
    color: var(--muted-2);
  }
  .save-indicator.error {
    color: var(--error);
  }
  .version {
    font-size: 11px;
    color: var(--muted-2);
  }
</style>
