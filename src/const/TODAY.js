import int from "@3-/int";

const DAY = 864e5,
  loop = () => {
    TODAY = int(Date.now() / DAY);
    setTimeout(loop, DAY - (Date.now() % DAY) + 100).unref();
  };

export let TODAY;
loop();
