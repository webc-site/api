import orgDb from "../../../db/orgDb.js";
import { KIND_TABLE } from "./KIND.js";
import { tokenKvRm } from "./key.js";

export default async (org_id, kind, rel_id, id) => {
  const [rec] = await orgDb(org_id)(
    "DELETE ONLY type::record('token',$id) WHERE rel=type::record('" +
      KIND_TABLE[kind] +
      "',$rel_id) RETURN BEFORE;",
    { id, rel_id }
  );

  if (!rec) return;

  await tokenKvRm(org_id, rel_id, id);
  return true;
};
