import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "./KV.js";
import SDB from "./SDB.js";

export default (sql) => async (k, param) => {
  const [
    {
      id: { id }
    }
  ] = await SDB(sql, param);
  await KV.set(k, u64Buf(id));
  return id;
};
