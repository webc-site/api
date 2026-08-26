export const MAIL = "webc.site@googlegroups.com",
  NAME = "Name",
  HOST = "webc.site",
  PASSWORD = "password",
  ALIAS_HOST_LI =
    process.env.NODE_ENV === "production"
      ? []
      : ["localhost", "127.0.0.1", "0.0.0.0", "::1", "[::1]"];
