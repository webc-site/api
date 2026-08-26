// GEN BY gen.js
import { string, uint64 } from "@1-/proto/E.js";
import { int32 } from "@1-/proto/D.js";
import req from "../_req.js";

export default (uid, mail, newCode, oldCode) =>
  req(4, [uint64, string, string, string], [int32], uid, mail, newCode, oldCode);
