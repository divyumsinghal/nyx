const baseConfig = require("@nyx/config/tailwind.config");
const contentPaths = require("@nyx/config/tailwind-content");

module.exports = {
  ...baseConfig,
  content: contentPaths,
  presets: [require("nativewind/preset")],
};
