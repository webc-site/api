import int from "@3-/int";
import Lru from "../../../lib/Lru.js";
import SDB from "../../../db/SDB.js";
import parse from "./parse.js";

const SQL =
    "UPSERT ONLY userAgent SET browser=$browser,browserVer=$browser_ver,os=$os,osVer=$os_ver WHERE browser=$browser AND browserVer=$browser_ver AND os=$os AND osVer=$os_ver",
  LRU = Lru(1e4);

export default async (req) => {
  const li = await parse(req),
    key = li.join("\0");

  let id = LRU.get(key);
  if (id) return id;

  const [browser, browser_ver, os, os_ver] = li,
    [
      {
        id: { id: rec_id }
      }
    ] = await SDB(SQL, {
      browser,
      browser_ver,
      os,
      os_ver
    });
  id = int(rec_id);
  LRU.set(key, id);
  return id;
};
