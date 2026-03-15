import { defineConfig } from 'vite';
import wasmPlugin from '@rollup/plugin-wasm';

const crossOriginHeaders = {
  'Cross-Origin-Opener-Policy':   'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

// Allow the deploy workflow to override the base for GitHub Pages.
// Default '/' works for local dev, Vite preview, and CI test builds.
const base = process.env.BASE_URL ?? '/';

export default defineConfig({
  base,
  plugins: [wasmPlugin()],
  resolve: { alias: { '@': '/src' } },
  server: {
    headers: crossOriginHeaders,
  },
  preview: {
    headers: crossOriginHeaders,
  },
  worker: {
    format: 'es',
    plugins: () => [wasmPlugin()],
    rollupOptions: {
      // /pkg/sim_engine.js is served from public/ at runtime; don't bundle it.
      external: (id) => /\/pkg\/sim_engine/.test(id),
    },
  },
  build: {
    target: 'esnext',
    assetsInlineLimit: 0,
    rollupOptions: {
      // Same for the main bundle — WASM is loaded dynamically at runtime from public/.
      external: (id) => /\/pkg\/sim_engine/.test(id),
    },
  },
});
