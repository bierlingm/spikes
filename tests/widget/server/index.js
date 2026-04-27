/**
 * Minimal static fixture server for widget tests.
 * Serves fixtures/*.html and resolves /spikes.js to the canonical widget.
 */

const http = require('http');
const fs = require('fs');
const path = require('path');

const PORT = 4717;
const REPO_ROOT = path.resolve(__dirname, '..', '..', '..');
const FIXTURES_DIR = path.join(__dirname, '..', 'fixtures');
const WIDGET_PATH = path.join(REPO_ROOT, 'widget', 'spikes.js');

const MIME_TYPES = {
  '.html': 'text/html',
  '.js': 'application/javascript',
  '.json': 'application/json',
};

const server = http.createServer((req, res) => {
  const url = new URL(req.url, `http://localhost:${PORT}`);
  const pathname = url.pathname;

  // Route /spikes.js to canonical widget
  if (pathname === '/spikes.js') {
    fs.readFile(WIDGET_PATH, 'utf-8', (err, data) => {
      if (err) {
        res.writeHead(500, { 'Content-Type': 'text/plain' });
        res.end('Error loading widget');
        return;
      }
      res.writeHead(200, { 'Content-Type': 'application/javascript' });
      res.end(data);
    });
    return;
  }

  // Serve fixtures/*.html
  let filePath = path.join(FIXTURES_DIR, pathname === '/' ? 'index.html' : pathname);
  const ext = path.extname(filePath).toLowerCase() || '.html';
  const contentType = MIME_TYPES[ext] || 'text/plain';

  fs.readFile(filePath, (err, data) => {
    if (err) {
      if (err.code === 'ENOENT') {
        res.writeHead(404, { 'Content-Type': 'text/plain' });
        res.end('Not found');
      } else {
        res.writeHead(500, { 'Content-Type': 'text/plain' });
        res.end('Server error');
      }
      return;
    }
    res.writeHead(200, { 'Content-Type': contentType });
    res.end(data);
  });
});

server.listen(PORT, () => {
  console.log(`Fixture server running at http://localhost:${PORT}`);
});

// Handle graceful shutdown
process.on('SIGTERM', () => {
  server.close(() => {
    process.exit(0);
  });
});
