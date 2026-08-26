#!/usr/bin/env bun

import { join } from "node:path";
import { readdirSync, existsSync } from "node:fs";
import kvInit from "./kv/init.js";
import sdbInit from "./sdb/init.js";
import confInit from "../conf/docker/init.js";

const SRC = join(import.meta.dirname, "../src"),
  main = async () => {
    await kvInit();
    await sdbInit();
    await confInit();
    for (const f of readdirSync(SRC, { withFileTypes: true })) {
      if (f.isDirectory()) {
        const file = join(SRC, f.name, "init.js");
        if (existsSync(file)) {
          try {
            const init = (await import(file)).default;
            if (init) await init();
          } catch (e) {
            console.error(e);
          }
        }
      }
    }
  };

export default main;

if (import.meta.main) {
  await main();
  process.exit(0);
}
