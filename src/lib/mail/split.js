import hostDecode from "../hostDecode.js";

export default (mail) => {
  mail = mail.trim();
  const at = mail.lastIndexOf("@");
  if (at <= 0) return [];
  return [
    mail.slice(0, at).slice(0, 255).toLowerCase(),
    hostDecode(mail.slice(at + 1).slice(0, 255))
  ];
};
