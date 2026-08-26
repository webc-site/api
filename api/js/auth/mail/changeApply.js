// GEN BY gen.js
import { string, uint64 } from "@1-/proto/E.js";
import { bool, int32 } from "@1-/proto/D.js";
import req from "../_req.js";

export default (uid, mail) => req(5, [uint64, string], [int32, bool], uid, mail);
