import { defineConfig } from "vite";
import glsl from "vite-plugin-glsl";

export default defineConfig(({ mode }) => ({
  base: "/floralcraft/",
  plugins: [
    glsl({
      include: ["**/*.wgsl"],
      watch: true,
      minify: mode === "production",
      removeDuplicatedImports: true,
    }),
  ],
}));
