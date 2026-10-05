import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

const host = process.env.TAURI_DEV_HOST;

// Tauri drives this dev server; the port and the src-tauri exclusion are not optional.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    // 1420 is unusable here: Windows has it inside the reserved range 1335-1434.
    port: 5183,
    strictPort: true,
    host: host || "127.0.0.1",
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
