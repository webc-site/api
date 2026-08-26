// GEN BY gen.js
import { string, uint64 } from "@1-/proto/E.js";
import req from "../_req.js";

export default (uid, id, name) => req(13, [uint64, uint64, string], [], uid, id, name);
