import { orgUserAccountLi, orgUserNameLi } from "../org/orgUser.js";
import { bidLi } from "./bid.js";

export default async (org_id, bid) => {
  const bid_user_li = await bidLi(org_id, bid),
    id_li = bid_user_li.map(([id]) => id),
    [name_li, account_li] = await Promise.all([
      orgUserNameLi(org_id, id_li),
      orgUserAccountLi(org_id, id_li)
    ]);

  return bid_user_li.map(([id, is_login], idx) => [
    id,
    name_li[idx] || "",
    is_login,
    account_li[idx] || [0, ""]
  ]);
};
