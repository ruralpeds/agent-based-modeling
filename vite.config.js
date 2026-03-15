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
  },
  build: {
    target: 'esnext',
    assetsInlineLimit: 0,
  },
});
