const path = require("path");

/**
 * Shared Babel config for Expo + NativeWind apps under Maya/.
 * @param {import("@babel/core").ConfigAPI} api
 * @param {{ projectRoot: string }} options projectRoot should be `__dirname` of the app's babel.config.js
 */
function createExpoNativeWindBabelConfig(api, { projectRoot }) {
  api.cache(true);
  const workspaceRoot = path.resolve(projectRoot, "../..");

  return {
    presets: [["babel-preset-expo", { jsxImportSource: "nativewind" }]],
    plugins: [
      [
        "module-resolver",
        {
          root: [projectRoot],
          alias: {
            "@nyx/ui": path.join(workspaceRoot, "packages/ui/src/index.ts"),
            "@nyx/api": path.join(workspaceRoot, "packages/api/src/index.ts"),
            "@": path.join(projectRoot, "src"),
          },
        },
      ],
    ],
  };
}

module.exports = { createExpoNativeWindBabelConfig };
