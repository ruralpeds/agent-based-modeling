/**
 * Cross-Origin Isolation service worker.
 * Injects COOP + COEP headers on every response so that SharedArrayBuffer
 * is available on GitHub Pages (which cannot set HTTP headers directly).
 *
 * Registration: index.html registers this before the main module script.
 * Scope: the directory this file is served from (i.e. the app root).
 */

self.addEventListener('install', () => self.skipWaiting());

self.addEventListener('activate', event =>
  event.waitUntil(self.clients.claim())
);

self.addEventListener('fetch', function (event) {
  // Skip opaque requests that would break with CORS
  if (event.request.cache === 'only-if-cached' && event.request.mode !== 'same-origin') {
    return;
  }

  event.respondWith(
    fetch(event.request)
      .then(function (response) {
        // Pass through error/opaque responses unchanged
        if (response.status === 0) return response;

        const headers = new Headers(response.headers);
        headers.set('Cross-Origin-Opener-Policy',   'same-origin');
        headers.set('Cross-Origin-Embedder-Policy', 'require-corp');
        headers.set('Cross-Origin-Resource-Policy', 'cross-origin');

        return new Response(response.body, {
          status:     response.status,
          statusText: response.statusText,
          headers,
        });
      })
      .catch(err => console.error('[coi-sw] fetch error:', err))
  );
});
