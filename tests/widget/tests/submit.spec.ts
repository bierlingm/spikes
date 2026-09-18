/**
 * Spikes.submit(): batch answers to POST /public/submissions, one
 * submission_id per call, reused across retries (reliable-intake).
 * All /public/* calls are mocked; nothing reaches spikes.sh.
 */

import { test, expect, Page } from '@playwright/test';

const PUBLIC = 'https://spikes.sh/public/';

async function mockSubmissions(page: Page, statuses: number[], posts: any[]) {
	await page.route(PUBLIC + '**', async (route, request) => {
		const url = new URL(request.url());
		if (request.method() === 'POST' && url.pathname === '/public/submissions') {
			const body = request.postDataJSON();
			posts.push(body);
			const status = statuses[Math.min(posts.length - 1, statuses.length - 1)];
			const payload = status < 300
				? { ok: true, submissionId: body.submission_id, answerCount: body.answers.length, ...(status === 200 ? { duplicate: true } : {}) }
				: { error: 'x', code: status === 429 ? 'RATE_LIMIT' : 'INTERNAL_ERROR', retry_after: 1 };
			await route.fulfill({ status, contentType: 'application/json', headers: status === 429 ? { 'Retry-After': '1' } : {}, body: JSON.stringify(payload) });
			return;
		}
		await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify({ data: [] }) });
	});
}

test.describe('Spikes.submit', () => {
	test.beforeEach(async ({ page }) => {
		await page.addInitScript(() => {
			localStorage.setItem('spikes:reviewer', JSON.stringify({ id: 'rev-submit', name: 'Submit Tester', email: 'hidden@example.com' }));
		});
	});

	test('VAL-SUBMIT-001: posts the answers with project, a UUID submission_id and the reviewer', async ({ page }) => {
		const posts: any[] = [];
		await mockSubmissions(page, [201], posts);
		await page.goto('/readback.html');
		await page.waitForFunction(() => !!(window as any).Spikes);

		const result = await page.evaluate(() => (window as any).Spikes.submit([
			{ question_id: 'q1', body: 'B' },
			{ key: 'budget', title: 'Budget', body: '40k' },
		]));

		expect(posts).toHaveLength(1);
		expect(posts[0].project).toBe('widget-ci-readback');
		expect(posts[0].submission_id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
		expect(posts[0].reviewer).toEqual({ id: 'rev-submit', name: 'Submit Tester' });
		expect(posts[0].url).toBe('http://localhost:4717/readback.html');
		expect(posts[0].answers).toHaveLength(2);
		expect(result).toEqual({ ok: true, submissionId: posts[0].submission_id, answerCount: 2, duplicate: false });
	});

	test('VAL-SUBMIT-002: retries 5xx and 429 with the same submission_id', async ({ page }) => {
		const posts: any[] = [];
		await mockSubmissions(page, [503, 429, 200], posts);
		await page.goto('/readback.html');
		await page.waitForFunction(() => !!(window as any).Spikes);

		const result = await page.evaluate(() => (window as any).Spikes.submit([{ key: 'k', title: 'T', body: 'x' }], { retries: 3 }));

		expect(posts).toHaveLength(3);
		expect(new Set(posts.map(p => p.submission_id)).size).toBe(1);
		expect(result.duplicate).toBe(true);
	});

	test('VAL-SUBMIT-003: a 4xx rejects without retrying and exposes status and submissionId', async ({ page }) => {
		const posts: any[] = [];
		await mockSubmissions(page, [400], posts);
		await page.goto('/readback.html');
		await page.waitForFunction(() => !!(window as any).Spikes);

		const error = await page.evaluate(() => (window as any).Spikes.submit([{ key: 'k', body: 'x' }], { submissionId: '11111111-2222-4333-8444-555555555555' })
			.then(() => null, (e: any) => ({ status: e.status, submissionId: e.submissionId })));

		expect(posts).toHaveLength(1);
		expect(posts[0].submission_id).toBe('11111111-2222-4333-8444-555555555555');
		expect(error).toEqual({ status: 400, submissionId: '11111111-2222-4333-8444-555555555555' });
	});
});
