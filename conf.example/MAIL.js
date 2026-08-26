#!/usr/bin/env bun

const SENDER = "i@webc.pub",
  PASSWORD = "TODO";

const fetchSend = async (sender, password, sender_name, to, title, txt, html) => {
    const res = await fetch("https://mail.webc.pub/send", {
      method: "POST",
      body: JSON.stringify([
        sender,
        password,
        sender_name,
        to || "",
        title || "",
        txt || "",
        html || ""
      ])
    });

    if (!res.ok) {
      throw new Error(`send mail failed ${res.status}: ${await res.text()}`);
    }

    return res;
  },
  send = async (host, to, title, txt, html) => {
    return fetchSend(SENDER, PASSWORD, host, to, title, txt, html);
  };

export default send;

if (import.meta.main) {
  await send(
    "webc.site",
    "your@mail.com",
    "测试邮件 " + new Date(),
    "这是一封测试邮件。",
    '<p>这是一封<b style="color:#0f0;">测试邮件</b>。</p>'
  );
}
