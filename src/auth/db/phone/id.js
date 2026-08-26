import kvU64 from "../../../db/kvU64.js";
import { keyPhone } from "./key.js";

export default (area, num) => kvU64(keyPhone(area, num));
