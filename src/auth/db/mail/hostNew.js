import hostDecode from "../../../lib/hostDecode.js";
import upsertLru from "../../../db/upsertLru.js";
import { keyHost } from "./key.js";

const [run, MAIL_HOST_ID] = upsertLru("UPSERT ONLY mailHost SET host=$host WHERE host=$host");

export { MAIL_HOST_ID };

export default (host) => {
  host = hostDecode(host);
  return run(keyHost(host), { host });
};
