import { uint64Li } from "@1-/proto/E.js";

const PREFIX = Buffer.from("phone:");

export const keyPhone = (area, num) => Buffer.concat([PREFIX, uint64Li([area, num])]);
