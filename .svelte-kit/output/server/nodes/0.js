

export const index = 0;
let component_cache;
export const component = async () => component_cache ??= (await import('../entries/pages/_layout.svelte.js')).default;
export const universal = {
  "ssr": false
};
export const universal_id = "src/routes/+layout.ts";
export const imports = ["_app/immutable/nodes/0.C0yx0l-6.js","_app/immutable/chunks/CGO84EJ7.js"];
export const stylesheets = ["_app/immutable/assets/0.DF7vAftp.css"];
export const fonts = [];
