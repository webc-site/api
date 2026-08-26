import { readdirSync } from "node:fs";

export default (dir) =>
  readdirSync(dir, { withFileTypes: true })
    .filter((f) => f.isDirectory())
    .map((f) => f.name);
