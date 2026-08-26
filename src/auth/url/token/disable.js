import { KIND_USER } from "../../db/token/KIND.js";
import { tokenDisable } from "../../db/token/turn.js";
import DisableE from "../../gen/token/DisableE.js";

export default async function (uid, id) {
  if (await this.hasUser(uid)) {
    await tokenDisable(await this.org_id, KIND_USER, uid, id);
  }
  return DisableE([]);
}
