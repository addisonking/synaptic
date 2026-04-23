#!/usr/bin/env bun
// bump-version.ts
// usage: bun run scripts/bump-version.ts [patch|minor|major]

import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const VALID_BUMPS = ['patch', 'minor', 'major'] as const;
type BumpType = (typeof VALID_BUMPS)[number];

function isValidBump(s: string): s is BumpType {
	return VALID_BUMPS.includes(s as BumpType);
}

function parseVersion(v: string): [number, number, number] {
	const parts = v.split('.').map(Number);
	if (parts.length !== 3 || parts.some((n) => Number.isNaN(n))) {
		throw new Error(`invalid version: ${v}`);
	}
	return parts as [number, number, number];
}

function bumpVersion(current: string, type: BumpType): string {
	const [major, minor, patch] = parseVersion(current);
	switch (type) {
		case 'major':
			return `${major + 1}.0.0`;
		case 'minor':
			return `${major}.${minor + 1}.0`;
		case 'patch':
			return `${major}.${minor}.${patch + 1}`;
	}
}

function readJson(path: string) {
	return JSON.parse(readFileSync(path, 'utf-8'));
}

function writeJson(path: string, data: unknown) {
	writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`);
}

function updatePackageJson(root: string, newVersion: string) {
	const path = join(root, 'package.json');
	const pkg = readJson(path);
	pkg.version = newVersion;
	writeJson(path, pkg);
	console.log(`  updated package.json -> ${newVersion}`);
}

function updateCargoToml(root: string, newVersion: string) {
	const path = join(root, 'src-tauri', 'Cargo.toml');
	let content = readFileSync(path, 'utf-8');
	content = content.replace(
		/^version\s*=\s*"[^"]+"/m,
		`version = "${newVersion}"`,
	);
	writeFileSync(path, content);
	console.log(`  updated src-tauri/Cargo.toml -> ${newVersion}`);
}

function updateTauriConf(root: string, newVersion: string) {
	const path = join(root, 'src-tauri', 'tauri.conf.json');
	const conf = readJson(path);
	conf.version = newVersion;
	writeJson(path, conf);
	console.log(`  updated src-tauri/tauri.conf.json -> ${newVersion}`);
}

async function run(cmd: string[], options?: { cwd?: string }) {
	const proc = Bun.spawn(cmd, {
		cwd: options?.cwd,
		stdout: 'inherit',
		stderr: 'inherit',
	});
	const exitCode = await proc.exited;
	if (exitCode !== 0) {
		throw new Error(`command failed: ${cmd.join(' ')}`);
	}
}

async function main() {
	const bumpType = process.argv[2];
	if (!bumpType || !isValidBump(bumpType)) {
		console.error('usage: bun run scripts/bump-version.ts [patch|minor|major]');
		process.exit(1);
	}

	const root = join(import.meta.dir, '..');
	const pkg = readJson(join(root, 'package.json'));
	const currentVersion = pkg.version;
	const newVersion = bumpVersion(currentVersion, bumpType);

	// check for uncommitted changes before modifying anything
	const statusProc = Bun.spawn(['git', 'status', '--porcelain'], {
		cwd: root,
		stdout: 'pipe',
	});
	const statusOut = await new Response(statusProc.stdout).text();

	if (statusOut.trim().length > 0) {
		console.error('error: working directory has uncommitted changes.');
		console.error('commit or stash them before running this script.');
		process.exit(1);
	}

	console.log(`bumping ${bumpType}: ${currentVersion} -> ${newVersion}`);

	updatePackageJson(root, newVersion);
	updateCargoToml(root, newVersion);
	updateTauriConf(root, newVersion);

	const tag = `v${newVersion}`;

	await run(['git', 'add', '-A'], { cwd: root });
	await run(['git', 'commit', '-m', `chore: bump version to ${newVersion}`], {
		cwd: root,
	});
	await run(['git', 'tag', '-a', tag, '-m', `release ${tag}`], { cwd: root });
	await run(['git', 'push', 'origin', 'HEAD'], { cwd: root });
	await run(['git', 'push', 'origin', tag], { cwd: root });

	console.log(
		`\ndone. pushed tag ${tag}. github actions will build the release.`,
	);
}

main().catch((err) => {
	console.error(err);
	process.exit(1);
});
