import { readFileSync } from "fs";
const text = readFileSync(new URL("../src/seo_copy.rs", import.meta.url), "utf8");
const blocks = [...text.matchAll(/pub const (\w+): &str = "([\s\S]*?)";/g)];
for (const [, name, body] of blocks) {
  const w = body.split(/\s+/).filter(Boolean).length;
  const kind = name.endsWith("_H1") ? "H1" : "DESC";
  console.log(`${name} ${kind} ${w}`);
}
