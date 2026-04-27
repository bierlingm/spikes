/**
 * Bytes-freshness marker checks for the canonical widget/spikes.js.
 * These are Node-level tests (no browser required).
 */

import { test, expect } from '@playwright/test';
import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

const REPO_ROOT = path.resolve(__dirname, '../../..');
const WIDGET_PATH = process.env.SCRATCH_WIDGET_PATH || path.join(REPO_ROOT, 'widget', 'spikes.js');

test.describe('Marker Checks', () => {
	test('VAL-MARKER-001: widget/spikes.js exists and is non-empty', () => {
		expect(fs.existsSync(WIDGET_PATH)).toBe(true);
		const stats = fs.statSync(WIDGET_PATH);
		expect(stats.size).toBeGreaterThan(0);
	});

	test('VAL-MARKER-002: spikes-error-dot occurs >= 1 time', () => {
		const content = fs.readFileSync(WIDGET_PATH, 'utf-8');
		const count = (content.match(/spikes-error-dot/g) || []).length;
		expect(count).toBeGreaterThanOrEqual(1);
	});

	test('VAL-MARKER-003: setErrorState occurs >= 4 times', () => {
		const content = fs.readFileSync(WIDGET_PATH, 'utf-8');
		const count = (content.match(/setErrorState/g) || []).length;
		expect(count).toBeGreaterThanOrEqual(4);
	});

	test('VAL-MARKER-004: console.error(\'[Spikes] Sync failed occurs >= 1 time', () => {
		const content = fs.readFileSync(WIDGET_PATH, 'utf-8');
		const count = (content.match(/console\.error\('\[Spikes\] Sync failed/g) || []).length;
		expect(count).toBeGreaterThanOrEqual(1);
	});

	test('VAL-MARKER-005: widget has > 100 lines (anti-minification guard)', () => {
		const content = fs.readFileSync(WIDGET_PATH, 'utf-8');
		const lines = content.split('\n').length;
		expect(lines).toBeGreaterThan(100);
	});

	test('VAL-MARKER-006: All markers can be detected via grep', () => {
		// Verify the same assertions work via shell grep (cross-check)
		const grepCount = (pattern: string): number => {
			try {
				const result = execSync(`grep -c "${pattern}" ${WIDGET_PATH}`, { encoding: 'utf-8' });
				return parseInt(result.trim(), 10);
			} catch (e) {
				return 0;
			}
		};

		// Fixed-string grep for patterns with special chars (parentheses, brackets)
		const grepFixedCount = (pattern: string): number => {
			try {
				const result = execSync(`grep -Fc "${pattern}" ${WIDGET_PATH}`, { encoding: 'utf-8' });
				return parseInt(result.trim(), 10);
			} catch (e) {
				return 0;
			}
		};

		expect(grepCount('spikes-error-dot')).toBeGreaterThanOrEqual(1);
		expect(grepCount('setErrorState')).toBeGreaterThanOrEqual(4);
		expect(grepFixedCount("console.error('[Spikes] Sync failed")).toBeGreaterThanOrEqual(1);
	});
});
