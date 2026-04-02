const { createExpoNativeWindBabelConfig } = require("@nyx/config/babel.config");

module.exports = function (api) {
  return createExpoNativeWindBabelConfig(api, { projectRoot: __dirname });
};
