#!/usr/bin/env bun

import { parseEnv } from "node:util";
import read from "@3-/read";
import { join } from "node:path";
import { URI, DB_CONF } from "../../conf/SDB.js";
import openDb from "../../src/db/sdb/open.js";
import kvInit from "../kv/init.js";
import KV from "../../src/db/KV.js";

const ROOT = import.meta.dirname,
  main = async () => {
    await kvInit();
    try {
      await KV.flushdb();
    } catch {}
    const env_file = join(ROOT, "../.env"),
      { SDB_PASSWORD } = parseEnv(read(env_file)),
      { namespace, database, username, password } = DB_CONF,
      open = await openDb(URI, "root", SDB_PASSWORD, namespace),
      db = open(),
      ns_db = open(database);

    await db("REMOVE NAMESPACE IF EXISTS " + namespace + ";");
    await db("DEFINE NAMESPACE " + namespace + ";");

    await ns_db("DEFINE DATABASE " + database + ";");
    await ns_db(
      "DEFINE USER OVERWRITE " + username + ' ON NAMESPACE PASSWORD "' + password + '" ROLES OWNER;'
    );

    const surql = read(join(ROOT, "sdb.surql")),
      stmt_li = surql
        .split(";")
        .map((s) => s.trim())
        .filter(Boolean);

    for (const stmt of stmt_li) {
      await ns_db(stmt + ";");
    }
    console.log("db namespace '" + namespace + "' reset & initialized");
  };

export default main;

if (import.meta.main) {
  await main();
  process.exit(0);
}
