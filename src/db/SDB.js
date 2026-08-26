import { DB_CONF, URI } from "../../conf/SDB.js";
import openDb from "./sdb/open.js";

const { username, password, namespace, database } = DB_CONF;

export const open = await openDb(URI, username, password, namespace);
export default open(database);
