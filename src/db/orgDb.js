import { open } from "./SDB.js";

export default (org_id) => open("org" + org_id);
