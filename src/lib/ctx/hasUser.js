import { bidUserHas } from "../../auth/db/user/bid.js";

export default (ctx) => async (uid) => bidUserHas(await ctx.org_id, ctx.bid, uid);
