import { getVersion } from '@tauri-apps/api/app';
import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';

export type UpdateStatus =
	| 'idle'
	| 'checking'
	| 'available'
	| 'downloading'
	| 'installing'
	| 'uptodate'
	| 'error';

export const updater = $state({
	status: 'idle' as UpdateStatus,
	version: '',
	body: '',
	error: null as string | null,
	progress: 0,
	appVersion: '',
});

export async function initAppVersion() {
	updater.appVersion = await getVersion();
}

export async function checkForUpdate() {
	updater.status = 'checking';
	updater.error = null;
	updater.version = '';
	updater.body = '';
	try {
		const update = await check();
		if (update) {
			updater.status = 'available';
			updater.version = update.version;
			updater.body = update.body || '';
			return update;
		} else {
			updater.status = 'uptodate';
			return null;
		}
	} catch (e) {
		updater.status = 'error';
		updater.error = String(e);
		return null;
	}
}

export async function downloadAndInstallUpdate() {
	updater.status = 'downloading';
	try {
		const update = await check();
		if (!update) {
			updater.status = 'uptodate';
			return;
		}
		await update.downloadAndInstall((event) => {
			switch (event.event) {
				case 'Started':
					updater.status = 'downloading';
					updater.progress = 0;
					break;
				case 'Progress':
					updater.status = 'downloading';
					updater.progress = event.data.percentage || 0;
					break;
				case 'Finished':
					updater.status = 'installing';
					break;
			}
		});
		await relaunch();
	} catch (e) {
		updater.status = 'error';
		updater.error = String(e);
	}
}

export function resetUpdater() {
	updater.status = 'idle';
	updater.error = null;
	updater.version = '';
	updater.body = '';
	updater.progress = 0;
}
