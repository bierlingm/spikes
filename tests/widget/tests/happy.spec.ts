/**
 * Happy path behavioral tests for the Spikes widget.
 * Tests the silent-404 scenario with mocked 200 responses.
 */

import { test, expect } from '@playwright/test';

test.describe('Happy Path - Silent 404', () => {
	// Pre-seed localStorage with reviewer before each test to skip name prompt
	test.beforeEach(async ({ page }) => {
		await page.addInitScript(() => {
			localStorage.setItem('spikes:reviewer', JSON.stringify({
				id: 'test-reviewer-happy',
				name: 'Happy Tester'
			}));
		});
	});

	test('VAL-HAPPY-001: data-project default endpoint resolves to https://spikes.sh/spikes', async ({ page }) => {
		// Track intercepted requests
		let interceptedUrl: string | null = null;
		let interceptedMethod: string | null = null;
		let interceptedHeaders: Record<string, string> = {};
		let interceptedBody: any = null;

		// Mock the spikes.sh endpoint with 200
		await page.route('https://spikes.sh/spikes', async (route, request) => {
			interceptedUrl = request.url();
			interceptedMethod = request.method();
			interceptedHeaders = await request.allHeaders();
			interceptedBody = await request.postDataJSON();
			await route.fulfill({
				status: 200,
				body: '{"success": true}'
			});
		});

		// Load the happy path fixture
		await page.goto('/happy.html');

		// Wait for widget to mount
		await page.waitForSelector('#spikes-btn');

		// First click: enter armed mode (force to bypass animation stability check)
		await page.click('#spikes-btn', { force: true });

		// Wait a moment for state transition
		await page.waitForTimeout(100);

		// Second click: open modal (page feedback)
		await page.click('#spikes-btn', { force: true });

		// Wait for modal to appear
		await page.waitForSelector('#spikes-modal', { state: 'visible' });

		// Select a rating
		await page.click('#spikes-ratings button[data-rating="like"]');

		// Enter comment
		await page.fill('#spikes-comments', 'This is a happy path test comment');

		// Click save
		await page.click('#spikes-save');

		// Wait a moment for async request
		await page.waitForTimeout(500);

		// Assert URL is exactly https://spikes.sh/spikes
		expect(interceptedUrl).toBe('https://spikes.sh/spikes');
		expect(interceptedMethod).toBe('POST');
		expect(interceptedHeaders['content-type']).toContain('application/json');

		// Assert body shape
		expect(interceptedBody).toBeTruthy();
		expect(interceptedBody.id).toBeTruthy();
		expect(typeof interceptedBody.id).toBe('string');
		expect(interceptedBody.id.length).toBeGreaterThan(0);
		expect(interceptedBody.type).toBe('page');
		expect(interceptedBody.projectKey).toBe('widget-ci-silent404');
		expect(interceptedBody.rating).toBe('like');
		expect(interceptedBody.comments).toBe('This is a happy path test comment');
		expect(interceptedBody.reviewer).toBeTruthy();
		expect(interceptedBody.reviewer.id).toBe('test-reviewer-happy');
		expect(interceptedBody.reviewer.name).toBe('Happy Tester');
		expect(interceptedBody.timestamp).toBeTruthy();
		expect(interceptedBody.viewport).toBeTruthy();
		expect(typeof interceptedBody.viewport.width).toBe('number');
		expect(typeof interceptedBody.viewport.height).toBe('number');

		// Verify ISO timestamp format
		expect(() => new Date(interceptedBody.timestamp)).not.toThrow();
	});

	test('VAL-HAPPY-002: POST uses correct method and Content-Type', async ({ page }) => {
		let interceptedMethod: string | null = null;
		let contentType: string | null = null;

		await page.route('https://spikes.sh/spikes', async (route, request) => {
			interceptedMethod = request.method();
			const headers = await request.allHeaders();
			contentType = headers['content-type'];
			await route.fulfill({ status: 200, body: '{}' });
		});

		await page.goto('/happy.html');
		await page.waitForSelector('#spikes-btn');

		// Use unique comment to avoid duplicate guard
		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-HAPPY-002 test comment');
		await page.click('#spikes-save');
		await page.waitForTimeout(500);

		expect(interceptedMethod).toBe('POST');
		expect(contentType).toContain('application/json');
	});

	test('VAL-HAPPY-003: Spike body shape is well-formed', async ({ page }) => {
		let interceptedBody: any = null;

		await page.route('https://spikes.sh/spikes', async (route, request) => {
			interceptedBody = await request.postDataJSON();
			await route.fulfill({ status: 200, body: '{}' });
		});

		await page.goto('/happy.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-HAPPY-003 body shape test');
		await page.click('#spikes-save');
		await page.waitForTimeout(500);

		// Check all required fields
		expect(interceptedBody).toMatchObject({
			type: 'page',
			projectKey: 'widget-ci-silent404',
			rating: 'like',
			comments: 'VAL-HAPPY-003 body shape test'
		});

		// Check id is non-empty string
		expect(interceptedBody.id).toBeTruthy();
		expect(typeof interceptedBody.id).toBe('string');
		expect(interceptedBody.id.length).toBeGreaterThan(0);

		// Check reviewer shape
		expect(interceptedBody.reviewer).toBeTruthy();
		expect(typeof interceptedBody.reviewer.id).toBe('string');
		expect(interceptedBody.reviewer.id.length).toBeGreaterThan(0);
		expect(typeof interceptedBody.reviewer.name).toBe('string');
		expect(interceptedBody.reviewer.name.length).toBeGreaterThan(0);

		// Check ISO 8601 timestamp
		expect(interceptedBody.timestamp).toMatch(/^\d{4}-\d{2}-\d{2}T/);
		const timestampDate = new Date(interceptedBody.timestamp);
		expect(timestampDate.getTime()).not.toBeNaN();

		// Check viewport
		expect(interceptedBody.viewport).toBeTruthy();
		expect(typeof interceptedBody.viewport.width).toBe('number');
		expect(typeof interceptedBody.viewport.height).toBe('number');
		expect(interceptedBody.viewport.width).toBeGreaterThan(0);
		expect(interceptedBody.viewport.height).toBeGreaterThan(0);
	});

	test('VAL-HAPPY-004: Mocked 200 leaves the error dot hidden', async ({ page }) => {
		await page.route('https://spikes.sh/spikes', async (route) => {
			await route.fulfill({ status: 200, body: '{}' });
		});

		await page.goto('/happy.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-HAPPY-004 error dot test');
		await page.click('#spikes-save');

		// Wait for request to complete
		await page.waitForTimeout(500);

		// Check error dot is hidden or not in DOM
		const errorDot = page.locator('#spikes-error-dot');
		const count = await errorDot.count();
		if (count > 0) {
			const isVisible = await errorDot.isVisible().catch(() => false);
			expect(isVisible).toBe(false);
		}
	});

	test('VAL-HAPPY-005: Mocked 200 leaves the success-state button title intact', async ({ page }) => {
		await page.route('https://spikes.sh/spikes', async (route) => {
			await route.fulfill({ status: 200, body: '{}' });
		});

		await page.goto('/happy.html');
		await page.waitForSelector('#spikes-btn');

		// Get initial title
		const initialTitle = await page.getAttribute('#spikes-btn', 'title');
		expect(initialTitle).not.toContain('Last feedback failed to sync');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-HAPPY-005 title test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Check title does not contain failure literal (title may be null or default after save)
		const finalTitle = await page.getAttribute('#spikes-btn', 'title');
		if (finalTitle !== null) {
			expect(finalTitle).not.toContain('Last feedback failed to sync');
		}
	});

	test('VAL-HAPPY-006: Exactly one POST per single submit', async ({ page }) => {
		let requestCount = 0;

		await page.route('https://spikes.sh/spikes', async (route) => {
			requestCount++;
			await route.fulfill({ status: 200, body: '{}' });
		});

		await page.goto('/happy.html');
		await page.waitForSelector('#spikes-btn');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'VAL-HAPPY-006 request count test');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		expect(requestCount).toBe(1);
	});

	test('VAL-HAPPY-007: Successful submit clears a pre-existing error indicator', async ({ page }) => {
		let requestCount = 0;

		await page.route('https://spikes.sh/spikes', async (route) => {
			requestCount++;
			// First request fails with 500, second succeeds with 200
			if (requestCount === 1) {
				await route.fulfill({
					status: 500,
					body: JSON.stringify({ error: 'Internal Server Error' })
				});
			} else {
				await route.fulfill({ status: 200, body: '{}' });
			}
		});

		await page.goto('/happy.html');
		await page.waitForSelector('#spikes-btn');

		// First submit - should fail with 500
		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'First submit - will fail');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Verify error state after 500
		const errorDotAfter500 = page.locator('#spikes-error-dot');
		if (await errorDotAfter500.count() > 0) {
			expect(await errorDotAfter500.isVisible()).toBe(true);
		}
		const titleAfter500 = await page.getAttribute('#spikes-btn', 'title');
		expect(titleAfter500).toContain('Last feedback failed to sync');

		// Second submit - need to use different comment to bypass 30s duplicate guard
		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal');
		await page.click('#spikes-ratings button[data-rating="like"]');
		await page.fill('#spikes-comments', 'Second submit - will succeed and clear error');
		await page.click('#spikes-save');

		await page.waitForTimeout(500);

		// Verify error state cleared after 200
		const errorDotAfter200 = page.locator('#spikes-error-dot');
		if (await errorDotAfter200.count() > 0) {
			expect(await errorDotAfter200.isVisible()).toBe(false);
		}
		const titleAfter200 = await page.getAttribute('#spikes-btn', 'title');
		expect(titleAfter200).not.toContain('Last feedback failed to sync');
	});
});
