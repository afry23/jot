import { defineConfig } from "vite";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [sveltekit(), tailwindcss()],
  theme: {
    extend: {
      colors: {
        tab: {
          yellow: '#F6C046',
          orange: '#F0874F',
          red: '#F25D66',
          gray: '#7D7E8A',
          blueGray: '#7E7E9A',
          purple: '#9D7E9A',
          green: '#7E9D7E',
        },
        // App theme colors
        primary: {
          DEFAULT: '#f0a05a', // Your accent color
          dark: '#d38a4a',
        },
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
