import { bidUserNow } from "../../auth/db/user/bid.js";

export default async (ctx) => bidUserNow(await ctx.org_id, ctx.bid);
