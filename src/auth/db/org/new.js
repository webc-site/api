import SDB from "../../../db/SDB.js";
import dbInit from "./dbInit.js";

export default async (host) => {
  const res = await SDB(
      "BEGIN TRANSACTION;LET $o=(CREATE ONLY org SET name=$name);CREATE ONLY host SET host=$name,org=$o.id;RETURN $o;COMMIT TRANSACTION;",
      { name: host }
    ),
    org_id = res.at(-2).id.id;
  await dbInit(org_id);
  return org_id;
};
