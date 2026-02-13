// Spikes Worker — spikes.sh feedback backend
// Cloudflare Worker + D1

interface Env {
  DB: D1Database;
  ASSETS: R2Bucket;
  SPIKES_TOKEN: string;
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
}

const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization',
};

function jsonResponse(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      'Content-Type': 'application/json',
      ...corsHeaders,
    },
  });
}

function errorResponse(message: string, status = 400): Response {
  return jsonResponse({ error: message }, status);
}

function validateToken(request: Request, env: Env): boolean {
  const url = new URL(request.url);
  const token = url.searchParams.get('token');
  return token === env.SPIKES_TOKEN;
}

function getBearerToken(request: Request): string | null {
  const auth = request.headers.get('Authorization');
  if (!auth?.startsWith('Bearer ')) return null;
  return auth.slice(7);
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
      return jsonResponse({ status: 'ok', service: 'spikes-sh-worker' });
    }

    // POST /spikes — public write (for widget)
    if (path === '/spikes' && request.method === 'POST') {
      return handleCreateSpike(request, env);
    }

    // GET /spikes — requires token (for CLI/admin)
    if (path === '/spikes' && request.method === 'GET') {
      if (!validateToken(request, env)) {
        return errorResponse('Unauthorized', 401);
      }
      return handleListSpikes(request, env);
    }

    // GET /spikes/:id — requires token
    const spikeIdMatch = path.match(/^\/spikes\/([^\/]+)$/);
    if (spikeIdMatch && request.method === 'GET') {
      if (!validateToken(request, env)) {
        return errorResponse('Unauthorized', 401);
      }
      return handleGetSpike(spikeIdMatch[1], env);
    }

    // GET /prospects — export emails (requires token)
    if (path === '/prospects' && request.method === 'GET') {
      if (!validateToken(request, env)) {
        return errorResponse('Unauthorized', 401);
      }
      return handleListProspects(env);
    }

    // POST /shares — create a share (bearer auth, multipart)
    if (path === '/shares' && request.method === 'POST') {
      const ownerToken = getBearerToken(request);
      if (!ownerToken) {
        return errorResponse('Unauthorized', 401);
      }
      return handleCreateShare(request, ownerToken, env);
    }

    // GET /shares — list user's shares (bearer auth)
    if (path === '/shares' && request.method === 'GET') {
      const ownerToken = getBearerToken(request);
      if (!ownerToken) {
        return errorResponse('Unauthorized', 401);
      }
      return handleListShares(ownerToken, env);
    }

    // DELETE /shares/:id — delete a share (bearer auth)
    const shareIdMatch = path.match(/^\/shares\/([^\/]+)$/);
    if (shareIdMatch && request.method === 'DELETE') {
      const ownerToken = getBearerToken(request);
      if (!ownerToken) {
        return errorResponse('Unauthorized', 401);
      }
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

  // Check spike limit if this spike belongs to a share
  const shareId = body.share_id ? String(body.share_id) : (body.projectKey ? String(body.projectKey) : null);
  if (shareId) {
    const share = await env.DB.prepare(
      'SELECT spike_count FROM shares WHERE id = ?'
    ).bind(shareId).first<{ spike_count: number }>();

    if (share && share.spike_count >= FREE_TIER_LIMITS.maxSpikesPerShare) {
      return jsonResponse({
        error: 'Spike limit reached for this share',
        code: 'SPIKE_LIMIT',
      }, 429);
    }
  }

  const spike: Spike = {
    id: String(body.id || crypto.randomUUID()),
    project: String(body.projectKey || body.project || 'spikes.sh'),
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
  };

  try {
    await env.DB.prepare(`
      INSERT INTO spikes (
        id, project, page, url, type, selector, xpath, element_text,
        bounding_box, rating, comments, reviewer_id, reviewer_name,
        reviewer_email, timestamp, viewport, user_agent
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).bind(
      spike.id,
      spike.project,
      spike.page,
      spike.url,
      spike.type,
      spike.selector,
      spike.xpath,
      spike.element_text,
      spike.bounding_box,
      spike.rating,
      spike.comments,
      spike.reviewer_id,
      spike.reviewer_name,
      spike.reviewer_email,
      spike.timestamp,
      spike.viewport,
      spike.user_agent
    ).run();

    // Increment spike count for the share
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

async function handleListSpikes(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url);
  const page = url.searchParams.get('page');
  const reviewer = url.searchParams.get('reviewer');
  const rating = url.searchParams.get('rating');
  const project = url.searchParams.get('project');

  let query = 'SELECT * FROM spikes WHERE 1=1';
  const params: string[] = [];

  if (project) {
    query += ' AND project = ?';
    params.push(project);
  }
  if (page) {
    query += ' AND page LIKE ?';
    params.push(`%${page}%`);
  }
  if (reviewer) {
    query += ' AND (reviewer_name LIKE ? OR reviewer_id = ?)';
    params.push(`%${reviewer}%`, reviewer);
  }
  if (rating) {
    query += ' AND rating = ?';
    params.push(rating);
  }

  query += ' ORDER BY timestamp DESC';

  try {
    const stmt = env.DB.prepare(query);
    const result = params.length > 0 
      ? await stmt.bind(...params).all<Spike>()
      : await stmt.all<Spike>();

    const spikes = (result.results || []).map(row => ({
      id: row.id,
      type: row.type,
      projectKey: row.project,
      page: row.page,
      url: row.url,
      reviewer: {
        id: row.reviewer_id,
        name: row.reviewer_name,
        email: row.reviewer_email,
      },
      selector: row.selector,
      xpath: row.xpath,
      elementText: row.element_text,
      boundingBox: row.bounding_box ? JSON.parse(row.bounding_box) : null,
      rating: row.rating,
      comments: row.comments,
      timestamp: row.timestamp,
      viewport: row.viewport ? JSON.parse(row.viewport) : null,
      userAgent: row.user_agent,
    }));

    return jsonResponse(spikes);
  } catch (e) {
    console.error('DB query error:', e);
    return errorResponse('Failed to fetch spikes', 500);
  }
}

async function handleGetSpike(id: string, env: Env): Promise<Response> {
  try {
    const result = await env.DB.prepare(
      'SELECT * FROM spikes WHERE id = ?'
    ).bind(id).first<Spike>();

    if (!result) {
      return errorResponse('Spike not found', 404);
    }

    const spike = {
      id: result.id,
      type: result.type,
      projectKey: result.project,
      page: result.page,
      url: result.url,
      reviewer: {
        id: result.reviewer_id,
        name: result.reviewer_name,
        email: result.reviewer_email,
      },
      selector: result.selector,
      xpath: result.xpath,
      elementText: result.element_text,
      boundingBox: result.bounding_box ? JSON.parse(result.bounding_box) : null,
      rating: result.rating,
      comments: result.comments,
      timestamp: result.timestamp,
      viewport: result.viewport ? JSON.parse(result.viewport) : null,
      userAgent: result.user_agent,
    };

    return jsonResponse(spike);
  } catch (e) {
    console.error('DB query error:', e);
    return errorResponse('Failed to fetch spike', 500);
  }
}

async function handleListProspects(env: Env): Promise<Response> {
  try {
    const result = await env.DB.prepare(`
      SELECT DISTINCT reviewer_email, reviewer_name, MIN(timestamp) as first_seen
      FROM spikes 
      WHERE reviewer_email IS NOT NULL AND reviewer_email != ''
      GROUP BY reviewer_email
      ORDER BY first_seen DESC
    `).all<{ reviewer_email: string; reviewer_name: string; first_seen: string }>();

    const prospects = (result.results || []).map(row => ({
      email: row.reviewer_email,
      name: row.reviewer_name,
      firstSeen: row.first_seen,
    }));

    return jsonResponse(prospects);
  } catch (e) {
    console.error('DB query error:', e);
    return errorResponse('Failed to fetch prospects', 500);
  }
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
    woff: 'font/woff2',
    woff2: 'font/woff2',
  };
  return types[ext] || 'application/octet-stream';
}

async function handleShareRoute(path: string, env: Env): Promise<Response> {
  const segments = path.slice(3).split('/'); // Remove '/s/'
  const slug = segments[0];
  if (!slug) {
    return errorResponse('Not Found', 404);
  }

  const share = await env.DB.prepare(
    'SELECT id, slug FROM shares WHERE slug = ?'
  ).bind(slug).first<{ id: string; slug: string }>();

  if (!share) {
    return errorResponse('Not Found', 404);
  }

  const filepath = segments.slice(1).join('/') || 'index.html';
  const r2Key = `shares/${share.id}/${filepath}`;

  const object = await env.ASSETS.get(r2Key);
  if (!object) {
    return errorResponse('Not Found', 404);
  }

  const contentType = getContentType(filepath);
  let body: ArrayBuffer | string = await object.arrayBuffer();

  if (contentType === 'text/html') {
    const html = new TextDecoder().decode(body);
    const widgetScript = `<script src="https://spikes.sh/widget.js" data-endpoint="https://spikes.sh" data-project="${share.id}"></script>`;
    body = html.replace('</body>', `${widgetScript}\n</body>`);
  }

  return new Response(body, {
    headers: {
      'Content-Type': contentType,
      ...corsHeaders,
    },
  });
}

interface ShareRow {
  id: string;
  slug: string;
  owner_token: string;
  created_at: string;
  spike_count: number;
  tier: string;
}

function generateSlug(name: string): string {
  const sanitized = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '').slice(0, 30);
  const suffix = Math.random().toString(36).slice(2, 7);
  return sanitized ? `${sanitized}-${suffix}` : suffix;
}

// Free tier limits
const FREE_TIER_LIMITS = {
  maxShares: 5,
  maxUploadBytes: 50 * 1024 * 1024, // 50MB
  maxSpikesPerShare: 1000,
};

async function handleCreateShare(request: Request, ownerToken: string, env: Env): Promise<Response> {
  try {
    // Check active shares count for free tier
    const countResult = await env.DB.prepare(
      'SELECT COUNT(*) as count FROM shares WHERE owner_token = ?'
    ).bind(ownerToken).first<{ count: number }>();

    if (countResult && countResult.count >= FREE_TIER_LIMITS.maxShares) {
      return jsonResponse({
        error: 'Share limit reached',
        code: 'SHARE_LIMIT',
        upgrade_url: 'https://spikes.sh/pro',
      }, 429);
    }

    const formData = await request.formData();
    const metadataField = formData.get('metadata');
    const metadata = metadataField ? JSON.parse(metadataField as string) : {};

    // Check total upload size
    let totalSize = 0;
    for (const [key, value] of formData.entries()) {
      if (key === 'metadata') continue;
      if (value instanceof File) {
        totalSize += value.size;
      }
    }
    if (totalSize > FREE_TIER_LIMITS.maxUploadBytes) {
      return jsonResponse({
        error: 'Upload too large',
        code: 'SIZE_LIMIT',
        max_bytes: FREE_TIER_LIMITS.maxUploadBytes,
      }, 413);
    }
    
    const shareId = crypto.randomUUID();
    const slug = generateSlug(metadata.name || 'share');
    const createdAt = new Date().toISOString();

    // Upload all files to R2
    const uploads: Promise<void>[] = [];
    for (const [key, value] of formData.entries()) {
      if (key === 'metadata') continue;
      if (value instanceof File) {
        const r2Key = `shares/${shareId}/${value.name}`;
        uploads.push(env.ASSETS.put(r2Key, await value.arrayBuffer()));
      }
    }
    await Promise.all(uploads);

    // Create D1 record
    await env.DB.prepare(
      'INSERT INTO shares (id, slug, owner_token, created_at, spike_count, tier) VALUES (?, ?, ?, ?, 0, ?)'
    ).bind(shareId, slug, ownerToken, createdAt, 'free').run();

    return jsonResponse({
      ok: true,
      url: `https://spikes.sh/s/${slug}`,
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

    const shares = (result.results || []).map(row => ({
      id: row.id,
      slug: row.slug,
      url: `https://spikes.sh/s/${row.slug}`,
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

    if (!share) {
      return errorResponse('Share not found', 404);
    }

    if (share.owner_token !== ownerToken) {
      return errorResponse('Forbidden', 403);
    }

    // Fetch spikes for export before deletion
    const spikesResult = await env.DB.prepare(
      'SELECT * FROM spikes WHERE share_id = ?'
    ).bind(id).all<Spike>();

    const exportedSpikes = (spikesResult.results || []).map(row => ({
      id: row.id,
      type: row.type,
      projectKey: row.project,
      page: row.page,
      url: row.url,
      reviewer: {
        id: row.reviewer_id,
        name: row.reviewer_name,
        email: row.reviewer_email,
      },
      selector: row.selector,
      xpath: row.xpath,
      elementText: row.element_text,
      boundingBox: row.bounding_box ? JSON.parse(row.bounding_box) : null,
      rating: row.rating,
      comments: row.comments,
      timestamp: row.timestamp,
      viewport: row.viewport ? JSON.parse(row.viewport) : null,
      userAgent: row.user_agent,
    }));

    // Delete R2 files under /shares/{id}/
    const prefix = `shares/${id}/`;
    const listed = await env.ASSETS.list({ prefix });
    if (listed.objects.length > 0) {
      await Promise.all(listed.objects.map(obj => env.ASSETS.delete(obj.key)));
    }

    // Delete spikes with this share_id
    await env.DB.prepare('DELETE FROM spikes WHERE share_id = ?').bind(id).run();

    // Delete share record
    await env.DB.prepare('DELETE FROM shares WHERE id = ?').bind(id).run();

    return jsonResponse({ ok: true, exported_spikes: exportedSpikes });
  } catch (e) {
    console.error('Delete share error:', e);
    return errorResponse('Failed to delete share', 500);
  }
}
