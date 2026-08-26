import SDB from "../../../db/SDB.js";

export default async (db, id) => {
  const [p] = await SDB("SELECT area,num FROM ONLY type::record('phone',$id)", { id });
  if (p) {
    const { area, num } = p;
    await db("UPSERT ONLY type::record('phone',$id) SET area=$area,num=$num", {
      id,
      area,
      num
    });
    return "+" + area + " " + num;
  }
};
