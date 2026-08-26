import SDB from "../../../db/SDB.js";
import gen, { BOOL, BYTES, INT, STRING, rec } from "../../../db/gen.js";

const AREA = "area",
  HOST = "host",
  PREFIX = "prefix",
  MAIL = "mail",
  PHONE = "phone",
  TOKEN = "token",
  TS = "ts",
  USER = "user",
  TEAM = "team",
  REL = "rel",
  INIT_SQL = gen({
    mailHost: {
      field: [[HOST, STRING]],
      unique: HOST
    },
    mail: {
      field: [
        [HOST, rec("mailHost")],
        [PREFIX, STRING]
      ],
      unique: [[HOST, PREFIX]]
    },
    phone: {
      field: [
        [AREA, INT],
        ["num", INT]
      ],
      unique: [[AREA, "num"]]
    },
    userAgent: {
      field: [
        ["browser", STRING],
        ["browserVer", STRING],
        ["os", STRING],
        ["osVer", STRING]
      ],
      unique: [["browser", "browserVer", "os", "osVer"]]
    },
    team: {
      autoId: REL,
      field: [
        ["name", STRING],
        ["owner", rec(USER)],
        [TS, INT]
      ],
      index: TS
    },
    user: {
      autoId: REL,
      field: [
        ["level", INT],
        [MAIL, rec(MAIL, 1)],
        ["name", STRING],
        ["password", BYTES + " DEFAULT <bytes> ''"],
        [PHONE, rec(PHONE, 1)],
        [TS, INT]
      ],
      unique: [MAIL, PHONE],
      index: TS
    },
    userBidSignined: {
      autoId: true,
      field: [
        [USER, rec(USER)],
        ["bid", BYTES],
        ["ip", BYTES],
        ["ua", rec("userAgent", 1)],
        [TS, INT]
      ],
      unique: [[USER, "bid"]],
      index: TS
    },
    userBidLog: {
      autoId: true,
      field: [
        [USER, rec(USER)],
        ["bid", BYTES],
        ["ip", BYTES],
        ["ua", rec("userAgent", 1)],
        ["action", INT],
        [TS, INT]
      ],
      index: [[USER, "action"], [USER, "bid"], "action", TS]
    },
    [TOKEN]: {
      autoId: true,
      field: [
        [REL, rec(USER + "|" + TEAM)],
        [TOKEN, BYTES],
        ["name", STRING + " DEFAULT ''"],
        ["enable", BOOL + " DEFAULT true"],
        [TS, INT]
      ],
      index: [[REL, TS]]
    }
  });

export default async (org_id) => {
  const org = "org" + org_id;
  await SDB("DEFINE DATABASE IF NOT EXISTS " + org + ";USE DB " + org + ";" + INIT_SQL);
};
