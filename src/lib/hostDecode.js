import { toUnicode } from "punycode";

export default (host) => toUnicode(host.toLowerCase().trim());
