import orgDb from "../../../db/orgDb.js";
import { KIND_TABLE } from "./KIND.js";
import { tokenInfoSet } from "./key.js";

export default async (org_id, kind, rel_id, id, name = "") => {
  name = name.slice(0, 24);
  const [rec] = await orgDb(org_id)(
    "UPDATE ONLY type::record('token',$id) SET name=$name WHERE rel=type::record('" +
      KIND_TABLE[kind] +
      "',$rel_id) RETURN token,enable,ts;",
    { id, rel_id, name }
  );

  if (!rec) return;

  const { token, enable, ts } = rec;
  await tokenInfoSet(org_id, id, [token, name, enable, ts]);
  return true;
};
