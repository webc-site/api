import sec from "@3-/time/sec.js";
import orgDb from "../../../db/orgDb.js";
import { KIND_TABLE } from "./KIND.js";
import tokenNew from "./_/token.js";
import { tokenKvSet } from "./key.js";

export default async (org_id, kind, rel_id, name = "") => {
  name = name.slice(0, 24);
  const token = tokenNew(),
    now = sec(),
    [{ id }] = await orgDb(org_id)(
      "CREATE ONLY token SET rel=type::record('" +
        KIND_TABLE[kind] +
        "',$rel_id),token=$val,name=$name,enable=true,ts=$ts;",
      {
        rel_id,
        val: token,
        name,
        ts: now
      }
    ),
    token_id = id.id,
    info = [token, name, true, now];

  await tokenKvSet(org_id, rel_id, token_id, info);

  return [token_id, info];
};
