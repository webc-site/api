import u64B255 from "@3-/intbin/u64B255.js";

const COLON = Buffer.from(":"),
  PREFIX_NAME = Buffer.from("orgUserName:"),
  PREFIX_LEVEL = Buffer.from("orgUserLevel:"),
  PREFIX_PASSWORD = Buffer.from("orgUserPassword:"),
  PREFIX_ACCOUNT = Buffer.from("orgUserAccount:"),
  PREFIX_MAIL = Buffer.from("orgUserMail:"),
  PREFIX_MAIL_USER = Buffer.from("orgMailUser:"),
  PREFIX_PHONE = Buffer.from("orgUserPhone:"),
  PREFIX_PHONE_USER = Buffer.from("orgPhoneUser:");

export const keyOrg = (prefix, org_id, suffix) =>
    Buffer.concat([prefix, u64B255(org_id), COLON, suffix]),
  keyHostOrg = (host) => "hostOrg:" + host,
  keyUserName = (org_id, uid_buf) => keyOrg(PREFIX_NAME, org_id, uid_buf),
  keyUserLevel = (org_id, uid_buf) => keyOrg(PREFIX_LEVEL, org_id, uid_buf),
  keyUserPassword = (org_id, uid_buf) => keyOrg(PREFIX_PASSWORD, org_id, uid_buf),
  keyUserAccount = (org_id, uid_buf) => keyOrg(PREFIX_ACCOUNT, org_id, uid_buf),
  keyUserMail = (org_id, uid_buf) => keyOrg(PREFIX_MAIL, org_id, uid_buf),
  keyOrgMailUser = (org_id, mail_id_buf) => keyOrg(PREFIX_MAIL_USER, org_id, mail_id_buf),
  keyUserPhone = (org_id, uid_buf) => keyOrg(PREFIX_PHONE, org_id, uid_buf),
  keyOrgPhoneUser = (org_id, phone_id_buf) => keyOrg(PREFIX_PHONE_USER, org_id, phone_id_buf);
