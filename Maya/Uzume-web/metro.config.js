const { getDefaultConfig } = require("expo/metro-config");
const { withNativeWind } = require("nativewind/metro");
const path = require("path");

const projectRoot = __dirname;
const workspaceRoot = path.resolve(projectRoot, "../..");

const config = getDefaultConfig(projectRoot);

// Watch all workspace packages
config.watchFolders = [workspaceRoot];

// Resolve from workspace packages first, then project
config.resolver.nodeModulesPaths = [
  path.resolve(projectRoot, "node_modules"),
  path.resolve(workspaceRoot, "node_modules"),
];

// Allow symlinked packages
config.resolver.disableHierarchicalLookup = false;

module.exports = withNativeWind(config, {
  input: "./global.css",
  inlineRem: 16,
});
