import { encode, decode, APPLICATION_CBOR } from "./cbor.js";
import { log, errLog } from "./log.js";
import normVal from "./normVal.js";

export default async (rpc_url, auth, namespace, database, sql, params = {}) => {
  const db_name = database || namespace || "",
    start = performance.now(),
    req = async (force) => {
      const headers = {
        "Content-Type": APPLICATION_CBOR,
        Accept: APPLICATION_CBOR,
        Authorization: "Bearer " + (await auth(force))
      };

      if (namespace) headers["Surreal-NS"] = namespace;
      if (database) headers["Surreal-DB"] = database;

      return fetch(rpc_url, {
        method: "POST",
        headers,
        body: encode({
          id: "q",
          method: "query",
          params: [sql, normVal(params)]
        })
      });
    };

  try {
    let res = await req();
    if (res.status === 401) res = await req(true);

    if (res.status !== 200) {
      const err = await res.text();
      throw new Error(err);
    }

    const data = decode(new Uint8Array(await res.arrayBuffer()));

    if (data.error) throw new Error(data.error);
    if (!data.result) throw new Error(data);

    log(sql, db_name, performance.now() - start);

    return data.result.map((item) => {
      if (item.status === "ERR") throw new Error(item.result);
      return item.result;
    });
  } catch (e) {
    errLog(sql, db_name, performance.now() - start, e);
    throw e;
  }
};
