#!/usr/bin/env bun

import write from "@3-/write";
import { join } from "node:path";
import SDB from "../../src/db/SDB.js";

const ROOT = import.meta.dirname,
  cleanDef = (def) => {
    if (!def) return "";
    let s = def.trim();
    if (!s.endsWith(";")) s += ";";
    return s
      .replace(/\s+PERMISSIONS FULL(?=;)/g, "")
      .replace(/\s+PERMISSIONS NONE(?=;)/g, "")
      .replace(/(\bDEFINE TABLE \S+)\s+TYPE NORMAL\b/g, "$1");
  },
  pushDef = (line_li, obj, key_li) => {
    if (!obj) return;
    for (const key of key_li) {
      const dict = obj[key];
      if (dict) {
        for (const def of Object.values(dict)) {
          line_li.push(cleanDef(def));
        }
      }
    }
  },
  main = async () => {
    if (!SDB) return;

    const [db_info] = await SDB("INFO FOR DB;"),
      line_li = [];

    pushDef(line_li, db_info, ["sequences", "functions", "params"]);

    const table_li = Object.entries(db_info.tables),
      table_info_li = await Promise.all(
        table_li.map(async ([table_name, table_def]) => {
          const [tb_info] = await SDB("INFO FOR TABLE " + table_name + ";");
          return [table_def, tb_info];
        })
      );

    for (const [table_def, tb_info] of table_info_li) {
      line_li.push("");
      line_li.push(cleanDef(table_def));
      pushDef(line_li, tb_info, ["fields", "indexes", "events"]);
    }

    const surql_file = join(ROOT, "sdb.surql");
    write(surql_file, line_li.join("\n") + "\n");
    console.log("dumped " + surql_file);
  };

export default main;

if (import.meta.main) {
  await main();
  process.exit(0);
}
