// Spikes Worker — spikes.sh feedback backend
// Cloudflare Worker + D1

interface Env {
  DB: D1Database;
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
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
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
