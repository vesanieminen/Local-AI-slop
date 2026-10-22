import { defineConfig } from 'vite';

// All model/HDRI assets live in public/ (committed real files, no symlinks).
// base: './' makes the built index.html reference its hashed assets relatively,
// so the site works under the GitHub Pages subpath (https://…/<repo>/).
export default defineConfig({
  base: './',
  server: {
    port: 5173,
  },
  preview: {
    port: 5173,
  },
});
