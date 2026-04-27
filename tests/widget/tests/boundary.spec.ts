/**
 * Boundary test for the Spikes widget.
 * Tests data-project resolution with explicit data-endpoint and no data-project.
 */

import { test, expect } from '@playwright/test';

test.describe('Boundary - data-project Resolution', () => {
	// Pre-seed localStorage with reviewer before each test
	test.beforeEach(async ({ page }) => {
		await page.addInitScript(() => {
			localStorage.setItem('spikes:reviewer', JSON.stringify({
				id: 'test-reviewer-boundary',
				name: 'Boundary Tester'
			}));
		});
	});

	test('VAL-BOUNDARY-001: Explicit data-endpoint wins, POST URL matches exactly', async ({ page }) => {
		// Track all intercepted requests
		const requests: Array<{ url: string; method: string; body: any }> = [];
		let spikesShRequestCount = 0;

		// First set up the catch-all route with fallback (lowest priority)
		await page.route('**/*', async (route, request) => {
			const url = request.url();
			// Track POST requests to non-localhost hosts
			if (request.method() === 'POST' && !url.includes('localhost') && !url.includes('127.0.0.1')) {
				requests.push({
					url: url,
					method: request.method(),
					body: null
				});
			}
			// Use fallback() to let other routes handle this request if they can
			await route.fallback();
		});

		// Track any requests to spikes.sh (should be zero) - register AFTER catch-all so it takes precedence
		await page.route('https://spikes.sh/spikes', async (route, request) => {
			spikesShRequestCount++;
			await route.abort();
		});

		// Mock the localhost sink endpoint - register LAST so it has highest priority
		await page.route('http://localhost:4717/sink', async (route, request) => {
			requests.push({
				url: request.url(),
				method: request.method(),
				body: await request.postDataJSON()
			});
			await route.fulfill({ status: 200, body: '{}' });
		});

		await page.goto('/boundary.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-BOUNDARY-001 endpoint test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert exactly one request was made
		expect(requests.length).toBe(1);

		// Assert URL is exactly the explicit endpoint
		expect(requests[0].url).toBe('http://localhost:4717/sink');
		expect(requests[0].method).toBe('POST');

		// Assert zero requests to spikes.sh
		expect(spikesShRequestCount).toBe(0);
	});

	test('VAL-BOUNDARY-002: projectKey falls back to the document host', async ({ page }) => {
		let interceptedBody: any = null;

		await page.route('http://localhost:4717/sink', async (route, request) => {
			interceptedBody = await request.postDataJSON();
			await route.fulfill({ status: 200, body: '{}' });
		});

		await page.goto('/boundary.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-BOUNDARY-002 projectKey test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert projectKey is 'localhost' (fallback to location.hostname)
		expect(interceptedBody).toBeTruthy();
		expect(interceptedBody.projectKey).toBe('localhost');
	});

	test('VAL-BOUNDARY-003: Boundary fixture also exercises the success path', async ({ page }) => {
		await page.route('http://localhost:4717/sink', async (route) => {
			await route.fulfill({ status: 200, body: '{}' });
		});

		await page.goto('/boundary.html');
		await page.waitForSelector('#spikes-btn');

		// Get initial title to compare
		const initialTitle = await page.getAttribute('#spikes-btn', 'title');
		expect(initialTitle).not.toContain('Last feedback failed to sync');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-BOUNDARY-003 success path test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert error dot is hidden (success path leaves no error indicator)
		const errorDot = page.locator('#spikes-error-dot');
		if (await errorDot.count() > 0) {
			expect(await errorDot.isVisible()).toBe(false);
		}

		// Assert title does not contain failure literal (title may be null or default after save)
		const finalTitle = await page.getAttribute('#spikes-btn', 'title');
		if (finalTitle !== null) {
			expect(finalTitle).not.toContain('Last feedback failed to sync');
		}
	});

	test('VAL-BOUNDARY-004: No spurious request to the spikes.sh default', async ({ page }) => {
		let localhostRequestCount = 0;
		let spikesShRequestCount = 0;

		// Mock the localhost sink
		await page.route('http://localhost:4717/sink', async (route, request) => {
			localhostRequestCount++;
			await route.fulfill({ status: 200, body: '{}' });
		});

		// Mock and track spikes.sh
		await page.route('https://spikes.sh/spikes', async (route) => {
			spikesShRequestCount++;
			await route.abort();
		});

		await page.goto('/boundary.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-BOUNDARY-004 no spikes.sh test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Assert we hit localhost sink
		expect(localhostRequestCount).toBe(1);

		// Assert zero requests to spikes.sh
		expect(spikesShRequestCount).toBe(0);
	});
});
