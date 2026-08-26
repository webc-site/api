import upsertLru from "../../../db/upsertLru.js";
import mailHostNew from "./hostNew.js";
import { keyMail } from "./key.js";
import split from "../../../lib/mail/split.js";

const [run, MAIL_ID] = upsertLru(
  "UPSERT ONLY mail SET host=type::record('mailHost',$host),prefix=$prefix WHERE host=type::record('mailHost',$host) AND prefix=$prefix"
);

export { MAIL_ID };

export default async (mail) => {
  const [prefix, host_name] = split(mail);
  if (!prefix) return;

  const host_id = await mailHostNew(host_name);
  if (!host_id) return;

  return run(keyMail(host_id, prefix), { host: host_id, prefix });
};
