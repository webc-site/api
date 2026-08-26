#!/usr/bin/env bun

import { existsSync } from "node:fs";
import { join } from "node:path";
import tran from "@1-/tran";
import srcPkg from "./sh/srcPkg.js";
import tranMd from "./sh/tran/md.js";

const ROOT = import.meta.dirname,
  src_dir = join(ROOT, "src");

await tran(ROOT);

for (const pkg of srcPkg(src_dir)) {
  const i18n_dir = join(src_dir, pkg, "i18n");
  if (existsSync(i18n_dir)) {
    tranMd(i18n_dir);
  }
}
