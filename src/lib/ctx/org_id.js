import { hostOrg } from "../../auth/db/org/hostOrg.js";

export default async (ctx) => {
  const { host } = ctx,
    org = host && (await hostOrg(host));
  if (org === undefined) {
    throw new Error("No Org " + host);
  }
  return org;
};
