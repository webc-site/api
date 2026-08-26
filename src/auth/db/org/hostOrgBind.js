import SDB from "../../../db/SDB.js";
import { hostOrgSet } from "./hostOrg.js";

export default async (org_id, host) => {
  const [{ id }] = await SDB("CREATE ONLY host SET host=$host,org=type::record('org',$org_id)", {
    host,
    org_id
  });
  await hostOrgSet(host, org_id);
  return id.id;
};
