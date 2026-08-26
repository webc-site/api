const OPTIONS = "OPTIONS",
  ORIGIN = "Origin",
  TRUE = "true",
  AGE = "86400",
  METHODS = "POST,OPTIONS";

export default async (c, next) => {
  const origin = c.req.header("origin");
  if (origin) {
    c.header("access-control-allow-origin", origin);
    c.header("access-control-allow-credentials", TRUE);
    c.header("vary", ORIGIN, { append: true });
  }
  if (c.req.method === OPTIONS) {
    c.header("access-control-allow-methods", METHODS);
    c.header("access-control-allow-headers", c.req.header("access-control-request-headers") || "*");
    c.header("access-control-max-age", AGE);
    return c.body(null, 204);
  }
  await next();
};
