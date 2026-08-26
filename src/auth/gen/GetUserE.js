import { $ as $E, bool, string, uint64 } from "@1-/proto/E.js";
import auth$Account from "./AccountE.js";
export default $E([
  /* 1 id */ uint64,
  /* 2 name */ string,
  /* 3 is_login */ bool,
  /* 4 account */ auth$Account
]);
