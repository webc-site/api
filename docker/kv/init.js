#!/usr/bin/env bun

import { parseEnv } from "node:util";
import read from "@3-/read";
import { join } from "node:path";
import Redis from "ioredis";
import sleep from "@3-/sleep";
import tcpping from "@3-/tcpping";
import { PASSWORD, PASSWORD_TEST } from "../../conf/KV.js";

const ROOT = import.meta.dirname,
  waitPort = async (host, port) => {
    for (let i = 0; i < 5; ++i) {
      try {
        if (await tcpping(host, port, 1000)) return true;
      } catch {
        await sleep(1000);
      }
    }
  },
  main = async () => {
    const env_file = join(ROOT, "../.env"),
      { R_PASSWORD, R_PORT = 6666 } = parseEnv(read(env_file)),
      host = "127.0.0.1",
      port = Number(R_PORT);

    if (!(await waitPort(host, port))) return;

    const admin = new Redis({
        host,
        port,
        password: R_PASSWORD,
        lazyConnect: true
      }),
      ns_li = [
        ["dev", PASSWORD],
        ["test", PASSWORD_TEST]
      ];

    try {
      await admin.connect();
    } catch {
      admin.disconnect();
      return;
    }

    for (const [ns, token] of ns_li) {
      try {
        await admin.call("NAMESPACE", "SET", ns, token);
      } catch {
        try {
          await admin.call("NAMESPACE", "ADD", ns, token);
        } catch {}
      }
    }

    await admin.call("CONFIG", "REWRITE");
    await admin.quit();
    console.log("kv namespace initialized");
  };

export default main;

if (import.meta.main) {
  await main();
  process.exit(0);
}
