const EMPTY_IP = new Uint8Array(),
  PREFIX_IP_FAIL = Buffer.from("ipFail:");

export const keyIpFail = (ip = EMPTY_IP) => Buffer.concat([PREFIX_IP_FAIL, ip]);
