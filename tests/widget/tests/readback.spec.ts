/**
 * Read-back tests: existing spikes as status pins, version banner,
 * questions panel with answer posting, and the off switches.
 * All /public/* calls are mocked; nothing reaches spikes.sh.
 */

import { test, expect, Page } from '@playwright/test';

const PUBLIC = 'https://spikes.sh/public/';

const spikesPayload = {
	data: [
		{
			id: 'sp-open', type: 'element', selector: '#hero', xpath: null, elementText: 'Read-back fixture',
			rating: 'meh', comments: 'Headline is too small', status: 'open', addressedIn: null,
			reviewer: { id: 'test-reviewer-readback', name: 'Readback Tester' },
			timestamp: '2026-09-10T10:00:00.000Z', lastReply: null, version: 'v0.5'
		},
		{
			id: 'sp-addressed', type: 'element', selector: '#cta', xpath: null, elementText: 'Book now',
			rating: 'no', comments: 'Button colour clashes', status: 'addressed', addressedIn: 'v0.5',
			reviewer: { id: 'someone-else', name: 'Other Reviewer' },
			timestamp: '2026-09-09T10:00:00.000Z',
			lastReply: { id: 'r1', authorType: 'agent', authorName: 'Builder', body: 'Changed to brand blue', versionLabel: 'v0.5', createdAt: '2026-09-11T10:00:00.000Z' },
			version: 'v0.5'
		},
		{
			id: 'sp-wontdo', type: 'page', selector: null, xpath: null, elementText: null,
			rating: null, comments: 'Add a dark mode', status: 'wont_do', addressedIn: null,
			reviewer: { id: 'someone-else', name: 'Other Reviewer' },
			timestamp: '2026-09-08T10:00:00.000Z', lastReply: null, version: 'v0.5'
		}
	]
};

const versionsPayload = {
	data: [
		{ label: 'v0.4', urlPrefix: 'http://localhost:4717/v0-4/', notes: 'old', createdAt: '2026-09-01T00:00:00.000Z' },
		{ label: 'v0.5', urlPrefix: 'http://localhost:4717/', notes: 'Addresses comments 1 and 2', createdAt: '2026-09-11T00:00:00.000Z' }
	]
};

const questionsPayload = {
	data: [
		{ id: 'q1', title: 'Which hero photo?', body: 'A: pool, B: kitchen', createdAt: '2026-09-11T00:00:00.000Z' },
		{ id: 'q2', title: 'Keep the booking widget?', body: null, createdAt: '2026-09-11T00:00:00.000Z' }
	]
};

async function mockPublic(page: Page, calls: string[], answers: any[]) {
	await page.route(PUBLIC + '**', async (route, request) => {
		const url = new URL(request.url());
		calls.push(request.method() + ' ' + url.pathname + url.search);
		if (request.method() === 'POST' && url.pathname.startsWith('/public/questions/')) {
			answers.push({ url: request.url(), body: await request.postDataJSON() });
			await route.fulfill({ status: 201, contentType: 'application/json', body: JSON.stringify({ id: 'a1', questionId: 'q1', createdAt: '2026-09-15T00:00:00.000Z' }) });
			return;
		}
		let body: any = { data: [] };
		if (url.pathname === '/public/spikes') body = spikesPayload;
		if (url.pathname === '/public/versions') body = versionsPayload;
		if (url.pathname === '/public/questions') body = questionsPayload;
		await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(body) });
	});
}

test.describe('Read-back', () => {
	test.beforeEach(async ({ page }) => {
		await page.addInitScript(() => {
			localStorage.setItem('spikes:reviewer', JSON.stringify({ id: 'test-reviewer-readback', name: 'Readback Tester' }));
		});
	});

	test('VAL-READBACK-001: fetches spikes, versions and questions for the current URL', async ({ page }) => {
		const calls: string[] = [];
		await mockPublic(page, calls, []);
		await page.goto('/readback.html');
		await page.waitForSelector('.spikes-readback-pin');

		expect(calls).toContain('GET /public/spikes?project=widget-ci-readback&url=' + encodeURIComponent('http://localhost:4717/readback.html'));
		expect(calls).toContain('GET /public/versions?project=widget-ci-readback');
		expect(calls).toContain('GET /public/questions?project=widget-ci-readback');
	});

	test('VAL-READBACK-002: element spikes render as pins coloured by status; page spikes are listed', async ({ page }) => {
		await mockPublic(page, [], []);
		await page.goto('/readback.html');
		await page.waitForSelector('.spikes-readback-pin');

		const pins = page.locator('.spikes-readback-pin');
		await expect(pins).toHaveCount(2);
		await expect(page.locator('.spikes-readback-pin[data-spike-id="sp-open"]')).toHaveAttribute('data-status', 'open');
		await expect(page.locator('.spikes-readback-pin[data-spike-id="sp-addressed"]')).toHaveAttribute('data-status', 'addressed');
		await expect(page.locator('.spikes-readback-pin[data-spike-id="sp-open"]')).toHaveCSS('background-color', 'rgb(231, 76, 60)');
		await expect(page.locator('.spikes-readback-pin[data-spike-id="sp-addressed"]')).toHaveCSS('background-color', 'rgb(34, 197, 94)');

		// Page-level wont_do spike appears in the modal's "Earlier feedback" list, not as a pin
		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal', { state: 'visible' });
		const card = page.locator('#spikes-earlier .spikes-readback-card[data-spike-id="sp-wontdo"]');
		await expect(card).toBeVisible();
		await expect(card.locator('.spikes-status')).toHaveText("Won't do");
	});

	test('VAL-READBACK-003: pin click shows comment, status, addressedIn, last reply, and "yours"', async ({ page }) => {
		await mockPublic(page, [], []);
		await page.goto('/readback.html');
		await page.waitForSelector('.spikes-readback-pin');

		await page.click('.spikes-readback-pin[data-spike-id="sp-open"]');
		const pop = page.locator('#spikes-readback-popover');
		await expect(pop).toBeVisible();
		await expect(pop).toContainText('Headline is too small');
		await expect(pop.locator('.spikes-status')).toHaveText('Open');
		await expect(pop.locator('.spikes-yours')).toHaveText('(yours)');

		await page.click('.spikes-readback-pin[data-spike-id="sp-addressed"]');
		await expect(pop).toHaveCount(1);
		await expect(page.locator('#spikes-readback-popover .spikes-status')).toHaveText('Addressed in v0.5');
		await expect(page.locator('#spikes-readback-popover .spikes-reply')).toContainText('Changed to brand blue');
		await expect(page.locator('#spikes-readback-popover .spikes-yours')).toHaveCount(0);
	});

	test('VAL-READBACK-004: version chip and notes use the longest matching urlPrefix', async ({ page }) => {
		await mockPublic(page, [], []);
		await page.goto('/readback.html');
		await expect(page.locator('#spikes-version')).toHaveText('v0.5');

		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal', { state: 'visible' });
		await expect(page.locator('#spikes-version-note')).toContainText('v0.5');
		await expect(page.locator('#spikes-version-note')).toContainText('Addresses comments 1 and 2');
	});

	test('VAL-READBACK-005: questions badge, answer posting with reviewer identity, answered state persists', async ({ page }) => {
		const answers: any[] = [];
		await mockPublic(page, [], answers);
		await page.goto('/readback.html');
		await expect(page.locator('#spikes-questions-badge')).toHaveText('2 questions for you');

		await page.click('#spikes-questions-badge');
		await page.waitForSelector('#spikes-modal', { state: 'visible' });
		const q1 = page.locator('.spikes-question[data-question-id="q1"]');
		await expect(q1).toContainText('Which hero photo?');
		await q1.locator('.spikes-answer').fill('B, the kitchen');
		await q1.locator('.spikes-answer-send').click();
		await page.waitForTimeout(300);

		expect(answers.length).toBe(1);
		expect(answers[0].url).toBe('https://spikes.sh/public/questions/q1/answers');
		expect(answers[0].body).toEqual({ body: 'B, the kitchen', reviewer: { id: 'test-reviewer-readback', name: 'Readback Tester' } });
		await expect(q1.locator('.spikes-answered')).toBeVisible();
		await expect(page.locator('#spikes-questions-badge')).toHaveText('1 question for you');

		// Answered state survives a reload
		await page.reload();
		await expect(page.locator('#spikes-questions-badge')).toHaveText('1 question for you');
	});

	test('VAL-READBACK-006: data-readback="off" and data-questions="off" make no /public requests', async ({ page }) => {
		const calls: string[] = [];
		await mockPublic(page, calls, []);
		await page.goto('/readback-off.html');
		await page.waitForSelector('#spikes-btn');
		await page.waitForTimeout(500);

		expect(calls).toEqual([]);
		await expect(page.locator('.spikes-readback-pin')).toHaveCount(0);
		await expect(page.locator('#spikes-questions-badge')).toBeHidden();
		await expect(page.locator('#spikes-version')).toBeHidden();
	});

	test('VAL-READBACK-007: read-back failures are silent and do not block feedback', async ({ page }) => {
		await page.route(PUBLIC + '**', route => route.fulfill({ status: 500, body: 'boom' }));
		let posted = false;
		await page.route('https://spikes.sh/spikes', async route => { posted = true; await route.fulfill({ status: 201, body: '{"ok":true,"id":"x"}' }); });
		const errors: string[] = [];
		page.on('console', msg => { if (msg.type() === 'error') errors.push(msg.text()); });

		await page.goto('/readback.html');
		await page.waitForSelector('#spikes-btn');
		await page.click('#spikes-btn', { force: true });
		await page.waitForTimeout(100);
		await page.click('#spikes-btn', { force: true });
		await page.waitForSelector('#spikes-modal', { state: 'visible' });
		await page.fill('#spikes-comments', 'still works');
		await page.click('#spikes-save');
		await page.waitForTimeout(300);

		expect(posted).toBe(true);
		expect(errors.filter(e => e.includes('[Spikes]'))).toEqual([]);
	});
});
