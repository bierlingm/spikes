/**
 * Reverse path behavioral tests for the Spikes widget.
 * Tests error handling with mocked 403, 500, and network failures.
 */

import { test, expect } from '@playwright/test';

test.describe('Reverse Path - Error Handling', () => {
	// Pre-seed localStorage with reviewer before each test
	test.beforeEach(async ({ page }) => {
		await page.addInitScript(() => {
			localStorage.setItem('spikes:reviewer', JSON.stringify({
				id: 'test-reviewer-reverse',
				name: 'Reverse Tester'
			}));
		});
	});

	test('VAL-REVERSE-001: Mocked 403 ORIGIN_NOT_ALLOWED makes the error dot visible', async ({ page }) => {
		// Capture console errors
		const consoleErrors: string[] = [];
		page.on('console', (msg) => {
			if (msg.type() === 'error') {
				consoleErrors.push(msg.text());
			}
		});

		await page.route('https://spikes.sh/spikes', async (route) => {
			await route.fulfill({
				status: 403,
				body: JSON.stringify({ error: 'ORIGIN_NOT_ALLOWED' })
			});
		});

		await page.goto('/reverse.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-REVERSE-001 403 test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert error dot is visible
		const errorDot = page.locator('#spikes-error-dot');
		await expect(errorDot).toBeVisible();
	});

	test('VAL-REVERSE-002: Button title flips to the failure literal on 403', async ({ page }) => {
		await page.route('https://spikes.sh/spikes', async (route) => {
			await route.fulfill({
				status: 403,
				body: JSON.stringify({ error: 'ORIGIN_NOT_ALLOWED' })
			});
		});

		await page.goto('/reverse.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-REVERSE-002 title test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert title contains failure literal (em-dash in the literal)
		const title = await page.getAttribute('#spikes-btn', 'title');
		expect(title).toContain('Last feedback failed to sync');
	});

	test('VAL-REVERSE-003: console.error emits the HTTP 403 sync-failed line', async ({ page }) => {
		// Capture console errors
		const consoleErrors: string[] = [];
		page.on('console', (msg) => {
			if (msg.type() === 'error') {
				consoleErrors.push(msg.text());
			}
		});

		await page.route('https://spikes.sh/spikes', async (route) => {
			await route.fulfill({
				status: 403,
				body: JSON.stringify({ error: 'ORIGIN_NOT_ALLOWED' })
			});
		});

		await page.goto('/reverse.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-REVERSE-003 console error test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert console.error contains the expected message
		const matchingErrors = consoleErrors.filter(e =>
			e.match(/\[Spikes\] Sync failed \(HTTP 403\)/)
		);
		expect(matchingErrors.length).toBeGreaterThan(0);

		// Also verify the URL is in the error
		expect(consoleErrors.some(e => e.includes('https://spikes.sh/spikes'))).toBe(true);
	});

	test('VAL-REVERSE-004: Mocked 500 produces the same visible failure state', async ({ page }) => {
		// Use a unique project for 500 test to avoid duplicate guard with other tests
		await page.addInitScript(() => {
			// Clear storage for fresh test
			localStorage.removeItem('spikes:widget-ci-silent500');
			localStorage.removeItem('spikes:last-spike:widget-ci-silent500');
		});

		// Load fixture with unique data-project via URL param or modify the script
		await page.route('https://spikes.sh/spikes', async (route) => {
			await route.fulfill({
				status: 500,
				body: JSON.stringify({ error: 'Internal Server Error' })
			});
		});

		// Create a fixture with unique project for this 500 test
		await page.goto('/reverse.html');
		await page.evaluate(() => {
			// Override the projectKey for this test by modifying config
			const script = document.querySelector('script[data-project="widget-ci-reverse-test"]');
			if (script) {
				script.setAttribute('data-project', 'widget-ci-silent500');
			}
			// Reload the widget logic by re-inserting the script
			const widgetScript = document.querySelector('script[src="/spikes.js"]');
			if (widgetScript) {
				// Force config reset by clearing and re-setting
				localStorage.removeItem('spikes:widget-ci-silent500');
				localStorage.removeItem('spikes:last-spike:widget-ci-silent500');
			}
		});

		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-REVERSE-004 500 error test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert error dot is visible
		const errorDot = page.locator('#spikes-error-dot');
		await expect(errorDot).toBeVisible();

		// Assert title contains failure literal
		const title = await page.getAttribute('#spikes-btn', 'title');
		expect(title).toContain('Last feedback failed to sync');
	});

	test('VAL-REVERSE-005: console.error emits the HTTP 500 sync-failed line', async ({ page }) => {
		const consoleErrors: string[] = [];
		page.on('console', (msg) => {
			if (msg.type() === 'error') {
				consoleErrors.push(msg.text());
			}
		});

		await page.route('https://spikes.sh/spikes', async (route) => {
			await route.fulfill({
				status: 500,
				body: JSON.stringify({ error: 'Internal Server Error' })
			});
		});

		await page.goto('/reverse.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-REVERSE-005 500 console test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert console.error contains HTTP 500 message
		expect(consoleErrors.some(e => e.match(/\[Spikes\] Sync failed \(HTTP 500\)/))).toBe(true);
	});

	test('VAL-REVERSE-006: Network failure (no response) emits the network-error variant', async ({ page }) => {
		const consoleErrors: string[] = [];
		page.on('console', (msg) => {
			if (msg.type() === 'error') {
				consoleErrors.push(msg.text());
			}
		});

		// Abort the request to simulate network failure
		await page.route('https://spikes.sh/spikes', async (route) => {
			await route.abort('failed');
		});

		await page.goto('/reverse.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-REVERSE-006 network failure test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert error dot is visible
		const errorDot = page.locator('#spikes-error-dot');
		await expect(errorDot).toBeVisible();

		// Assert title contains failure literal
		const title = await page.getAttribute('#spikes-btn', 'title');
		expect(title).toContain('Last feedback failed to sync');

		// Assert console.error contains the network-error variant
		expect(consoleErrors.some(e => e.includes('[Spikes] Could not sync to endpoint'))).toBe(true);
	});
});
