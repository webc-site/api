import SDB from "../../../db/SDB.js";

export default async (db, id) => {
  const [u] = await SDB("SELECT * FROM ONLY type::record('userAgent',$id)", { id });
  if (u) {
    await db(
      "UPSERT ONLY type::record('userAgent',$id) SET browser=$browser,browserVer=$browser_ver,os=$os,osVer=$os_ver",
      {
        id,
        browser: u.browser,
        browser_ver: u.browserVer,
        os: u.os,
        os_ver: u.osVer
      }
    );
  }
};
