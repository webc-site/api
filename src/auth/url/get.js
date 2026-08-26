import { authType } from "../db/host/authType.js";
import bidUserLi from "../db/user/bidUserLi.js";
import GetE from "../gen/GetE.js";

export default async function () {
  const { host, org_id, bid } = this,
    auth_type_li = (host && (await authType(host))) || [],
    user_li = await bidUserLi(await org_id, bid);

  return GetE([auth_type_li, user_li]);
}
