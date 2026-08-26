import { $ as $D, bool, string, uint64 } from "@1-/proto/D.js";
import auth$Account from "./AccountD.js";
export default $D([
  /* 1 id */ uint64,
  /* 2 name */ string,
  /* 3 is_login */ bool,
  /* 4 account */ auth$Account
]);
