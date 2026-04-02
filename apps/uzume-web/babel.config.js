module.exports = function (api) {
  api.cache(true);
  return {
    presets: [
      ["babel-preset-expo", { jsxImportSource: "nativewind" }],
      "nativewind/babel",
    ],
    plugins: [
      [
        "module-resolver",
        {
          root: ["./"],
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
