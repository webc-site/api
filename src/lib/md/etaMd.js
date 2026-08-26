import ETA from "./ETA.js";
import mdI18n from "./i18n.js";

export default (path, file) => {
  const tpl = mdI18n(path, file);
  return (lang, data) => {
    return ETA.renderString(tpl(lang), data);
  };
};
