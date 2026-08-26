#!/usr/bin/env bun

import { mkdirSync } from "node:fs";
import { join } from "node:path";
import { $, cd } from "@3-/zx";
import pkg from "../package.json";

$.verbose = 1;

const ROOT = import.meta.dirname,
  NAME = pkg.name.split("/").pop(),
  main = async () => {
    cd(ROOT);
    mkdirSync(join(ROOT, "data/surreal"), { recursive: true });
    mkdirSync(join(ROOT, "data/kvrocks"), { recursive: true });
    const running = await $({
      quiet: true
    })`docker compose -p ${NAME} ps --filter status=running --format json`;
    if (running.toString().trim()) {
      console.log(NAME + " is already running.");
      return;
    }
    const arg_li = process.argv.slice(2);
    if (arg_li.length === 0) {
      arg_li.push("-d");
    }

    await $`docker compose -p ${NAME} up ${arg_li}`;
    return true;
  };

export default main;

if (import.meta.main) {
  if (await main()) {
    await (await import("./init.js")).default();
  }
  process.exit(0);
}
