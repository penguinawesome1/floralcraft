import { defineConfig } from "vite";
import viteWesl from "wesl-plugin/vite";

export default defineConfig({
  base: "/floralcraft/",
  plugins: [viteWesl()],
});
