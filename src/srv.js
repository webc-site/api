import { Hono } from "hono";
import utf8e from "@3-/utf8/utf8e.js";
import reqIter from "./lib/srv/reqIter.js";
import resChunk from "./lib/srv/resChunk.js";
import bid from "./lib/srv/bid.js";
import cors from "./lib/srv/cors.js";
import { OK, ERR, CAPTCHA } from "@1-/protoapi/STATUS.js";
import URL from "./url.js";
import logerr from "@3-/log/ERR.js";
import log from "@3-/log/YELLOW.js";
import reqCtx from "./lib/srv/ctx.js";

const app = new Hono();

app.use(cors);
app.use(bid);

app.post("/", async (c) => {
  const body = new Uint8Array(await c.req.arrayBuffer()),
    ctx = reqCtx(c),
    stream = new ReadableStream({
      async start(controller) {
        for (const [id, mod, index, call_bin] of reqIter(body)) {
          const item = URL[mod];

          if (!item) {
            controller.enqueue(resChunk(id, ERR, utf8e("No Mod " + mod)));
            continue;
          }

          const [CALL_LI, CallD] = item,
            func = CALL_LI[index],
            args = CallD(call_bin)[index];
          let res,
            status = OK;
          log(func, args);
          try {
            const fn = (await import(`./${mod}/url/${func}.js`)).default;
            res = await fn.apply(ctx, args);
          } catch (err) {
            if (err === CAPTCHA) {
              status = CAPTCHA;
            } else {
              logerr(func, "(", args, ")");
              console.error(err);
              status = ERR;
              res = utf8e(err.message || err.toString());
            }
          }

          controller.enqueue(resChunk(id, status, res));
        }
        controller.close();
      }
    });

  return c.body(stream);
});

export default app;
