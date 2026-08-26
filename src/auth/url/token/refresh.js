import { KIND_USER } from "../../db/token/KIND.js";
import tokenRefresh from "../../db/token/refresh.js";
import RefreshE from "../../gen/token/RefreshE.js";

export default async function (uid, id) {
  return RefreshE(
    (await this.hasUser(uid))
      ? (await tokenRefresh(await this.org_id, KIND_USER, uid, id)) || []
      : []
  );
}
