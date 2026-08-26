// GEN BY gen.js
import { string, uint64 } from "@1-/proto/E.js";
import { uint64 as D_uint64 } from "@1-/proto/D.js";
import auth$token$Info from "../proto/token/InfoD.js";
import req from "../_req.js";

export default (uid, name) => req(14, [uint64, string], [D_uint64, auth$token$Info], uid, name);
