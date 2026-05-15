import { beforeEach, describe, expect, it } from 'vitest';
import {
	appState,
	goBack,
	goForward,
	markSaved,
	openFile,
	setFileContent,
	setZoom,
} from '../store.svelte';

function reset() {
	appState.system = null;
	appState.openFilePath = null;
	appState.openFileContent = '';
	appState.isDirty = false;
	appState.saveStatus = 'saved';
	appState.history = [];
	appState.historyIdx = -1;
	appState.zoom = 100;
}

beforeEach(reset);

describe('appState', () => {
	it('has correct initial state', () => {
		expect(appState.system).toBeNull();
		expect(appState.openFilePath).toBeNull();
		expect(appState.openFileContent).toBe('');
		expect(appState.isDirty).toBe(false);
		expect(appState.zoom).toBe(100);
		expect(appState.history).toEqual([]);
	});
});

describe('openFile', () => {
	it('sets file path and content', () => {
		openFile('/notes/foo.md', '# Hello');
		expect(appState.openFilePath).toBe('/notes/foo.md');
		expect(appState.openFileContent).toBe('# Hello');
		expect(appState.isDirty).toBe(false);
		expect(appState.saveStatus).toBe('saved');
	});

	it('initializes history with the path', () => {
		openFile('/notes/bar.md', 'content');
		expect(appState.history).toEqual(['/notes/bar.md']);
		expect(appState.historyIdx).toBe(0);
	});

	it('appends to history and truncates forward on new file', () => {
		openFile('/notes/a.md', 'a');
		openFile('/notes/b.md', 'b');
		openFile('/notes/c.md', 'c');
		expect(appState.history).toEqual([
			'/notes/a.md',
			'/notes/b.md',
			'/notes/c.md',
		]);
		expect(appState.historyIdx).toBe(2);
	});

	it('does not push duplicate consecutive paths', () => {
		openFile('/notes/x.md', 'same');
		openFile('/notes/x.md', 'same');
		expect(appState.history).toEqual(['/notes/x.md']);
	});
});

describe('goBack / goForward', () => {
	it('goes back in history', () => {
		openFile('/notes/1.md', '1');
		openFile('/notes/2.md', '2');
		openFile('/notes/3.md', '3');

		goBack();
		expect(appState.historyIdx).toBe(1);
		expect(appState.openFilePath).toBe('/notes/2.md');

		goBack();
		expect(appState.historyIdx).toBe(0);
		expect(appState.openFilePath).toBe('/notes/1.md');
	});

	it('does not go back past 0', () => {
		openFile('/notes/1.md', '1');
		goBack();
		expect(appState.historyIdx).toBe(0);
	});

	it('goes forward in history', () => {
		openFile('/notes/1.md', '1');
		openFile('/notes/2.md', '2');
		openFile('/notes/3.md', '3');

		// go back twice
		goBack();
		goBack();
		expect(appState.historyIdx).toBe(0);

		goForward();
		expect(appState.historyIdx).toBe(1);

		goForward();
		expect(appState.historyIdx).toBe(2);
	});

	it('does not go forward past end', () => {
		openFile('/notes/1.md', '1');
		openFile('/notes/2.md', '2');
		goForward();
		expect(appState.historyIdx).toBe(1);
	});
});

describe('setFileContent', () => {
	it('marks dirty', () => {
		openFile('/notes/test.md', 'original');
		setFileContent('modified');
		expect(appState.openFileContent).toBe('modified');
		expect(appState.isDirty).toBe(true);
		expect(appState.saveStatus).toBe('unsaved');
	});
});

describe('markSaved', () => {
	it('clears dirty flag', () => {
		openFile('/notes/test.md', 'original');
		setFileContent('modified');
		markSaved();
		expect(appState.isDirty).toBe(false);
		expect(appState.saveStatus).toBe('saved');
	});
});

describe('setZoom', () => {
	it('clamps between 50 and 200', () => {
		setZoom(30);
		expect(appState.zoom).toBe(50);
		setZoom(250);
		expect(appState.zoom).toBe(200);
		setZoom(100);
		expect(appState.zoom).toBe(100);
	});
});
