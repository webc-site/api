// GEN BY gen.js
import { string } from "@1-/proto/E.js";
import { uint64 } from "@1-/proto/D.js";
import req from "../_req.js";

export default (mail, password) => req(7, [string, string], [uint64], mail, password);
