#!/usr/bin/env bun

import mailNew from "./db/mail/new.js";
import hostOrgCreate, { hostOrg } from "./db/org/hostOrg.js";
import hostOrgBind from "./db/org/hostOrgBind.js";
import orgUser from "./db/org/orgUser.js";
import orgMailUser from "./db/mail/orgUser.js";
import orgDb from "../db/orgDb.js";
import { OWNER } from "./db/org/LEVEL.js";
import { MAIL, NAME, HOST, PASSWORD, ALIAS_HOST_LI } from "../../conf/initUser.js";

const main = async () => {
  const org_id = await hostOrgCreate(HOST),
    id = await mailNew(MAIL);

  if (org_id && ALIAS_HOST_LI) {
    for (const host of ALIAS_HOST_LI) {
      if (host !== HOST && !(await hostOrg(host))) {
        await hostOrgBind(org_id, host);
      }
    }
  }

  if (id && org_id) {
    if (await orgMailUser(org_id, MAIL)) return;
    const db = orgDb(org_id),
      user_id = await orgUser(db, org_id, OWNER, NAME, { password: PASSWORD, mail: id });
    console.log(user_id, org_id);
  }
};

export default main;

if (import.meta.main) {
  await main();
  process.exit(0);
}
