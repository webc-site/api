export default (tld, name, value, max_age, is_http_only = true) =>
  name +
  "=" +
  value +
  ";Path=/;Domain=" +
  tld +
  ";SameSite=None;Secure;Partitioned" +
  (max_age !== undefined ? ";Max-Age=" + max_age : "") +
  (is_http_only ? ";HttpOnly" : "");
