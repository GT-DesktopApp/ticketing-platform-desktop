import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { fileURLToPath, URL } from "node:url";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/  — tuned for Tauri desktop development.
export default defineConfig(async () => ({
  plugins: [react()],

  // Resolve the "@/..." path alias declared in tsconfig.
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  // 1. Don't let Vite hide Rust errors during `tauri dev`.
  clearScreen: false,
  // 2. Tauri needs a fixed dev port.
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      // Tauri watches src-tauri itself.
      ignored: ["**/src-tauri/**"],
    },
  },
  // 3. Produce builds compatible with the WebView shipped by Tauri.
  build: {
    target: "es2021",
    minify: "esbuild",
    sourcemap: false,
  },
}));
