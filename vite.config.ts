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
        note: resolve(__dirname, 'src/note.html'),
        taskboard: resolve(__dirname, 'src/taskboard.html'),
        todo: resolve(__dirname, 'src/todo.html'),
      },
    },
  },
}));
