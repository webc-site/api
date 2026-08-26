import orgDb from "../../../db/orgDb.js";
import { KIND_TABLE } from "./KIND.js";
import { tokenInfoSet } from "./key.js";

const _turn = (enable) => async (org_id, kind, rel_id, id) => {
  const [rec] = await orgDb(org_id)(
    "UPDATE ONLY type::record('token',$id) SET enable=$enable WHERE rel=type::record('" +
      KIND_TABLE[kind] +
      "',$rel_id) RETURN token,name,ts;",
    { id, rel_id, enable }
  );

  if (!rec) return;

  const { token, name, ts } = rec;
  await tokenInfoSet(org_id, id, [token, name, enable, ts]);
  return true;
};

export const tokenEnable = _turn(true),
  tokenDisable = _turn(false);
