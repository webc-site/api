import md2htm from "@1-/md2htm";

const B_CSS =
  "display:inline-block;padding:8px;color:#000;border-radius:8px;font-family:ui-monospace,monospace;font-size:1rem;font-weight:800;border:2px solid #000;letter-spacing:2px";

export default (md) =>
  md2htm(md)
    .replace(/<(b|strong)\b[^>]*>/gi, '<b style="' + B_CSS + '">')
    .replace(/<\/(b|strong)>/gi, "</b>");
