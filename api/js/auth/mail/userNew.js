// GEN BY gen.js
import { string } from "@1-/proto/E.js";
import { int32, uint64 } from "@1-/proto/D.js";
import req from "../_req.js";

export default (mail, name, password, verifyCode) =>
  req(8, [string, string, string, string], [uint64, int32], mail, name, password, verifyCode);
