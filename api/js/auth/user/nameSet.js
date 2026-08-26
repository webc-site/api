// GEN BY gen.js
import { string, uint64 } from "@1-/proto/E.js";
import req from "../_req.js";

export default (uid, name) => req(20, [uint64, string], [], uid, name);
