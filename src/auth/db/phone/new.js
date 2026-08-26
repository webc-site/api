import upsert from "../../../db/upsert.js";
import { keyPhone } from "./key.js";

const run = upsert("UPSERT ONLY phone SET area=$area,num=$num WHERE area=$area AND num=$num");

export default (area, num) => run(keyPhone(area, num), { area, num });
