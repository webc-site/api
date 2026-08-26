import { KIND_USER } from "../../db/token/KIND.js";
import { tokenEnable } from "../../db/token/turn.js";
import EnableE from "../../gen/token/EnableE.js";

export default async function (uid, id) {
  if (await this.hasUser(uid)) {
    await tokenEnable(await this.org_id, KIND_USER, uid, id);
  }
  return EnableE([]);
}
