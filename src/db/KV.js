import Redis from "ioredis";
import CONF from "../../conf/KV.js";

export default new Redis(CONF);
