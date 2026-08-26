import int from "@3-/int";
import { gray } from "@3-/log/GRAY.js";
import { blue } from "@3-/log/BLUE.js";
import greenLog from "@3-/log/GREEN.js";
import redLog from "@3-/log/RED.js";

const prefix = (sql, db, ms) => [gray(int(ms) + "ms"), blue(db), sql];

export const log = (sql, db, ms) => {
    greenLog(...prefix(sql, db, ms));
  },
  errLog = (sql, db, ms, err) => {
    redLog(...prefix(sql, db, ms), err.message || err);
  };
