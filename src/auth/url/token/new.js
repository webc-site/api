import { KIND_USER } from "../../db/token/KIND.js";
import tokenNew from "../../db/token/new.js";
import NewE from "../../gen/token/NewE.js";

export default async function (uid, name) {
  return NewE(
    (await this.hasUser(uid)) ? await tokenNew(await this.org_id, KIND_USER, uid, name) : []
  );
}
