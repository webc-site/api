#!/usr/bin/env bun

import { $, cd } from "@3-/zx";
import pkg from "../package.json";

$.verbose = 1;

const NAME = pkg.name.split("/").pop(),
  main = async () => {
    cd(import.meta.dirname);
    const arg_li = process.argv.slice(2);
    await $`docker compose -p ${NAME} down ${arg_li}`;
  };

export default main;

if (import.meta.main) {
  await main();
}
