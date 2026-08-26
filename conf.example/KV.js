export const HOST = "127.0.0.1",
  PORT = 6666,
  PASSWORD = "TODO",
  PASSWORD_TEST = "TODO_TEST";

export default {
  host: HOST,
  port: PORT,
  password: process.env.NODE_ENV === "test" ? PASSWORD_TEST : PASSWORD,
  lazyConnect: true
};
