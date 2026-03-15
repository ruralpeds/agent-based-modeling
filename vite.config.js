import { defineConfig } from 'vite';
import wasmPlugin from '@rollup/plugin-wasm';

const crossOriginHeaders = {
  'Cross-Origin-Opener-Policy':   'same-origin',
  'Cross-Origin-Embedder-Policy': 'require-corp',
};

export default defineConfig({
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
