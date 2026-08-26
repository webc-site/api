import u64Buf from "@3-/intbin/u64Buf.js";
import KV from "../../../db/KV.js";
import kvU64 from "../../../db/kvU64.js";
import { keyHostOrg } from "./key.js";
import orgNew from "./new.js";

export const hostOrg = async (host) => kvU64(keyHostOrg(host)),
  hostOrgSet = async (host, org_id) => {
    await KV.set(keyHostOrg(host), u64Buf(org_id));
    return org_id;
  };

export default async (host) => {
  let org_id = await hostOrg(host);
  if (org_id) return org_id;

  org_id = await orgNew(host);
  await hostOrgSet(host, org_id);
  return org_id;
};
