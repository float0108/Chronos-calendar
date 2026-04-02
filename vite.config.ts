import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        settings: resolve(__dirname, 'src/settings.html'),
        board: resolve(__dirname, 'src/board.html'),
        note: resolve(__dirname, 'src/note.html'),
        task: resolve(__dirname, 'src/task.html'),
        search: resolve(__dirname, 'src/search.html'),
      },
    },
  },
}));
