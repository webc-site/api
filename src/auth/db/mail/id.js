import kvU64 from "../../../db/kvU64.js";
import mailHostId from "./host.js";
import { keyMail } from "./key.js";
import { MAIL_ID } from "./new.js";
import split from "../../../lib/mail/split.js";

export default async (mail) => {
  const [prefix, host_name] = split(mail);
  if (!prefix) return;

  const host_id = await mailHostId(host_name);
  if (!host_id) return;

  const k = keyMail(host_id, prefix);
  let id = MAIL_ID.get(k);
  if (id) return id;

  id = await kvU64(k);
  if (id) MAIL_ID.set(k, id);
  return id;
};
