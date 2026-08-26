#!/usr/bin/env bun

import { parseEnv } from "node:util";
import read from "@3-/read";
import { join } from "node:path";
import { URI, DB_CONF } from "../../conf/SDB.js";
import openDb from "../../src/db/sdb/open.js";

const ROOT = import.meta.dirname,
  main = async () => {
    const env_file = join(ROOT, "../.env"),
      { SDB_PASSWORD } = parseEnv(read(env_file)),
      { namespace, database, username, password } = DB_CONF,
      open = await openDb(URI, "root", SDB_PASSWORD, namespace),
      db = open(),
      ns_db = open(database);

    await db("DEFINE NAMESPACE IF NOT EXISTS " + namespace + ";");
    await ns_db("DEFINE DATABASE IF NOT EXISTS " + database + ";");

    const [info] = await ns_db("INFO FOR DB;");
    if (info && Object.keys(info.tables || {}).length > 0) {
      console.log("db '" + database + "' already initialized");
      return;
    }
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
    console.log("db '" + database + "' initialized");
  };

export default main;

if (import.meta.main) {
  await main();
  process.exit(0);
}
