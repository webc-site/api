import sec from "@3-/time/sec.js";
import orgDb from "../../../db/orgDb.js";
import { KIND_TABLE } from "./KIND.js";
import tokenNew from "./_/token.js";
import { tokenKvSet } from "./key.js";

export default async (org_id, kind, rel_id, id) => {
  const token = tokenNew(),
    now = sec(),
    [rec] = await orgDb(org_id)(
      "UPDATE ONLY type::record('token',$id) SET token=$val,ts=$ts WHERE rel=type::record('" +
        KIND_TABLE[kind] +
        "',$rel_id) RETURN name,enable;",
      { id, rel_id, val: token, ts: now }
    );

  if (!rec) return;

  const { name, enable } = rec;
  await tokenKvSet(org_id, rel_id, id, [token, name, enable, now]);

  return [token, now];
};
