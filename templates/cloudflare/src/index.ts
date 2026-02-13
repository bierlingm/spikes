// Spikes Self-Host Worker — Minimal feedback backend
// Cloudflare Worker + D1 + R2

interface Env {
  DB: D1Database;
  ASSETS: R2Bucket;
}

interface Spike {
  id: string;
  project: string;
  page: string;
  url: string;
  type: string;
  selector: string | null;
  xpath: string | null;
  element_text: string | null;
  bounding_box: string | null;
  rating: string | null;
  comments: string;
  reviewer_id: string;
  reviewer_name: string;
  reviewer_email: string | null;
  timestamp: string;
  viewport: string | null;
  user_agent: string | null;
  share_id: string | null;
}

interface ShareRow {
  id: string;
  slug: string;
  owner_token: string;
  created_at: string;
  spike_count: number;
}

const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization',
};

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: { 'Content-Type': 'application/json', ...corsHeaders },
  });
}

function errorResponse(message: string, status = 400): Response {
  return jsonResponse({ error: message }, status);
}

function getBearerToken(request: Request): string | null {
  const auth = request.headers.get('Authorization');
  if (!auth?.startsWith('Bearer ')) return null;
  return auth.slice(7);
}

function generateSlug(name: string): string {
  const sanitized = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 30);
  const suffix = Math.random().toString(36).slice(2, 7);
  return sanitized ? `${sanitized}-${suffix}` : suffix;
}

function getContentType(filepath: string): string {
  const ext = filepath.split('.').pop()?.toLowerCase() || '';
  const types: Record<string, string> = {
    html: 'text/html',
    css: 'text/css',
    js: 'application/javascript',
    json: 'application/json',
    png: 'image/png',
    jpg: 'image/jpeg',
    jpeg: 'image/jpeg',
    gif: 'image/gif',
    svg: 'image/svg+xml',
    woff: 'font/woff',
    woff2: 'font/woff2',
  };
  return types[ext] || 'application/octet-stream';
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    if (request.method === 'OPTIONS') {
      return new Response(null, { status: 204, headers: corsHeaders });
    }

    const url = new URL(request.url);
    const path = url.pathname;

    // Health check
    if (path === '/' || path === '/health') {
      return jsonResponse({ status: 'ok', service: 'spikes-self-host' });
    }

    // POST /spikes — create spike (public)
    if (path === '/spikes' && request.method === 'POST') {
      return handleCreateSpike(request, env);
    }

    // POST /shares — create share (bearer auth)
    if (path === '/shares' && request.method === 'POST') {
      const ownerToken = getBearerToken(request);
      if (!ownerToken) return errorResponse('Unauthorized', 401);
      return handleCreateShare(request, ownerToken, env);
    }

    // GET /shares — list shares (bearer auth)
    if (path === '/shares' && request.method === 'GET') {
      const ownerToken = getBearerToken(request);
      if (!ownerToken) return errorResponse('Unauthorized', 401);
      return handleListShares(ownerToken, env);
    }

    // DELETE /shares/:id — delete share (bearer auth)
    const shareIdMatch = path.match(/^\/shares\/([^\/]+)$/);
    if (shareIdMatch && request.method === 'DELETE') {
      const ownerToken = getBearerToken(request);
      if (!ownerToken) return errorResponse('Unauthorized', 401);
      return handleDeleteShare(shareIdMatch[1], ownerToken, env);
    }

    // GET /s/* — serve shared projects
    if (path.startsWith('/s/')) {
      return handleShareRoute(path, env);
    }

    return errorResponse('Not Found', 404);
  },
};

async function handleCreateSpike(request: Request, env: Env): Promise<Response> {
  let body: Record<string, unknown>;
  try {
    body = await request.json();
  } catch {
    return errorResponse('Invalid JSON');
  }

  const shareId = body.share_id ? String(body.share_id) : (body.projectKey ? String(body.projectKey) : null);

  const spike: Spike = {
    id: String(body.id || crypto.randomUUID()),
    project: String(body.projectKey || body.project || 'default'),
    page: String(body.page || ''),
    url: String(body.url || ''),
    type: String(body.type || 'page'),
    selector: body.selector ? String(body.selector) : null,
    xpath: body.xpath ? String(body.xpath) : null,
    element_text: body.elementText ? String(body.elementText) : null,
    bounding_box: body.boundingBox ? JSON.stringify(body.boundingBox) : null,
    rating: body.rating ? String(body.rating) : null,
    comments: String(body.comments || ''),
    reviewer_id: body.reviewer?.id ? String(body.reviewer.id) : 'anon',
    reviewer_name: body.reviewer?.name ? String(body.reviewer.name) : 'Anonymous',
    reviewer_email: body.reviewer?.email ? String(body.reviewer.email) : null,
    timestamp: String(body.timestamp || new Date().toISOString()),
    viewport: body.viewport ? JSON.stringify(body.viewport) : null,
    user_agent: request.headers.get('User-Agent'),
    share_id: shareId,
  };

  try {
    await env.DB.prepare(`
      INSERT INTO spikes (
        id, project, page, url, type, selector, xpath, element_text,
        bounding_box, rating, comments, reviewer_id, reviewer_name,
        reviewer_email, timestamp, viewport, user_agent, share_id
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).bind(
      spike.id, spike.project, spike.page, spike.url, spike.type,
      spike.selector, spike.xpath, spike.element_text, spike.bounding_box,
      spike.rating, spike.comments, spike.reviewer_id, spike.reviewer_name,
      spike.reviewer_email, spike.timestamp, spike.viewport, spike.user_agent,
      spike.share_id
    ).run();

    if (shareId) {
      await env.DB.prepare(
        'UPDATE shares SET spike_count = spike_count + 1 WHERE id = ?'
      ).bind(shareId).run();
    }

    return jsonResponse({ ok: true, id: spike.id }, 201);
  } catch (e) {
    console.error('DB insert error:', e);
    return errorResponse('Failed to save spike', 500);
  }
}

async function handleCreateShare(request: Request, ownerToken: string, env: Env): Promise<Response> {
  try {
    const formData = await request.formData();
    const metadataField = formData.get('metadata');
    const metadata = metadataField ? JSON.parse(metadataField as string) : {};

    const shareId = crypto.randomUUID();
    const slug = generateSlug(metadata.name || 'share');
    const createdAt = new Date().toISOString();

    // Upload files to R2
    const uploads: Promise<void>[] = [];
    for (const [key, value] of formData.entries()) {
      if (key === 'metadata') continue;
      if (value instanceof File) {
        const r2Key = `shares/${shareId}/${value.name}`;
        uploads.push(env.ASSETS.put(r2Key, await value.arrayBuffer()));
      }
    }
    await Promise.all(uploads);

    await env.DB.prepare(
      'INSERT INTO shares (id, slug, owner_token, created_at, spike_count) VALUES (?, ?, ?, ?, 0)'
    ).bind(shareId, slug, ownerToken, createdAt).run();

    const host = new URL(request.url).origin;
    return jsonResponse({
      ok: true,
      url: `${host}/s/${slug}`,
      share_id: shareId,
      slug,
    }, 201);
  } catch (e) {
    console.error('Create share error:', e);
    return errorResponse('Failed to create share', 500);
  }
}

async function handleListShares(ownerToken: string, env: Env): Promise<Response> {
  try {
    const result = await env.DB.prepare(
      'SELECT id, slug, created_at, spike_count FROM shares WHERE owner_token = ? ORDER BY created_at DESC'
    ).bind(ownerToken).all<ShareRow>();

    const host = ''; // Will be filled by CLI
    const shares = (result.results || []).map(row => ({
      id: row.id,
      slug: row.slug,
      url: `/s/${row.slug}`,
      spike_count: row.spike_count,
      created_at: row.created_at,
    }));

    return jsonResponse(shares);
  } catch (e) {
    console.error('DB query error:', e);
    return errorResponse('Failed to fetch shares', 500);
  }
}

async function handleDeleteShare(id: string, ownerToken: string, env: Env): Promise<Response> {
  try {
    const share = await env.DB.prepare(
      'SELECT id, owner_token FROM shares WHERE id = ?'
    ).bind(id).first<ShareRow>();

    if (!share) return errorResponse('Share not found', 404);
    if (share.owner_token !== ownerToken) return errorResponse('Forbidden', 403);

    // Delete R2 files
    const prefix = `shares/${id}/`;
    const listed = await env.ASSETS.list({ prefix });
    if (listed.objects.length > 0) {
      await Promise.all(listed.objects.map(obj => env.ASSETS.delete(obj.key)));
    }

    await env.DB.prepare('DELETE FROM spikes WHERE share_id = ?').bind(id).run();
    await env.DB.prepare('DELETE FROM shares WHERE id = ?').bind(id).run();

    return jsonResponse({ ok: true });
  } catch (e) {
    console.error('Delete share error:', e);
    return errorResponse('Failed to delete share', 500);
  }
}

async function handleShareRoute(path: string, env: Env): Promise<Response> {
  const segments = path.slice(3).split('/');
  const slug = segments[0];
  if (!slug) return errorResponse('Not Found', 404);

  const share = await env.DB.prepare(
    'SELECT id, slug FROM shares WHERE slug = ?'
  ).bind(slug).first<{ id: string; slug: string }>();

  if (!share) return errorResponse('Not Found', 404);

  const filepath = segments.slice(1).join('/') || 'index.html';
  const r2Key = `shares/${share.id}/${filepath}`;

  const object = await env.ASSETS.get(r2Key);
  if (!object) return errorResponse('Not Found', 404);

  const contentType = getContentType(filepath);
  let body: ArrayBuffer | string = await object.arrayBuffer();

  if (contentType === 'text/html') {
    const html = new TextDecoder().decode(body);
    const widgetScript = `<script src="https://spikes.sh/widget.js" data-project="${share.id}"></script>`;
    body = html.replace('</body>', `${widgetScript}\n</body>`);
  }

  return new Response(body, {
    headers: { 'Content-Type': contentType, ...corsHeaders },
  });
}
