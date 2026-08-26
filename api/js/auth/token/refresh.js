// GEN BY gen.js
import { uint64 } from "@1-/proto/E.js";
import { bytes, uint64 as D_uint64 } from "@1-/proto/D.js";
import req from "../_req.js";

export default (uid, id) => req(15, [uint64, uint64], [bytes, D_uint64], uid, id);
