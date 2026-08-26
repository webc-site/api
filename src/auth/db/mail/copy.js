import SDB from "../../../db/SDB.js";

export default async (db, id) => {
  const [m] = await SDB(
    "SELECT prefix,host.id.id AS host_id,host.host AS host FROM ONLY type::record('mail',$id)",
    { id }
  );
  if (m) {
    await db(
      "UPSERT ONLY type::record('mailHost',$host_id) SET host=$host;UPSERT ONLY type::record('mail',$id) SET host=type::record('mailHost',$host_id),prefix=$prefix",
      {
        host_id: m.host_id,
        host: m.host,
        id,
        prefix: m.prefix
      }
    );
    return m.prefix + "@" + m.host;
  }
};
