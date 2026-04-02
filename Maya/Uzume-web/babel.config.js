module.exports = function (api) {
  api.cache(true);
  return {
    presets: [
      ["babel-preset-expo", { jsxImportSource: "nativewind" }],
    ],
    plugins: [
      [
        "module-resolver",
        {
          root: [__dirname],
          alias: {
            "@nyx/ui": "../../packages/ui/src/index.ts",
            "@nyx/api": "../../packages/api/src/index.ts",
            "@": "./src",
          },
        },
      ],
    ],
  };
};
