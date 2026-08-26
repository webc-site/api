// GEN BY gen.js
import { string } from "@1-/proto/E.js";
import { int32 } from "@1-/proto/D.js";
import req from "../_req.js";

export default (mail, password, verifyCode) =>
  req(21, [string, string, string], [int32], mail, password, verifyCode);
