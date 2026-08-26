#!/usr/bin/env bun

import { join } from "node:path";
import read from "@3-/read";
import SDB from "../../src/db/SDB.js";

const ROOT = import.meta.dirname,
  main = async () => {
    if (!SDB) return;

    const surql = read(join(ROOT, "sdb.surql")),
      stmt_li = surql
        .split(";")
        .map((s) => s.trim())
        .filter(Boolean);

    let count = 0;
    for (const stmt of stmt_li) {
      try {
        await SDB(stmt + ";");
        ++count;
      } catch (err) {
        if (err && err.message && err.message.includes("already exists")) {
          continue;
        }
        console.error("Error executing statement:", stmt);
        throw err;
      }
    }

    if (count > 0) {
      console.log("schema updated (" + count + " statement(s) applied)");
    } else {
      console.log("schema is already up to date");
    }
  };

export default main;

if (import.meta.main) {
  await main();
  process.exit(0);
}
