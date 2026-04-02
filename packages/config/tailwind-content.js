/**
 * Default Tailwind `content` globs for Maya Expo apps (run with cwd = app root).
 * Keeps @nyx/ui in the graph; includes ./src even when empty so new files are scanned.
 */
module.exports = [
  "./app/**/*.{js,jsx,ts,tsx}",
  "./src/**/*.{js,jsx,ts,tsx}",
  "../../packages/ui/src/**/*.{js,jsx,ts,tsx}",
];
