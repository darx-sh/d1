const core = globalThis.Deno.core;

class Logger {
  constructor(name) {
    this.name = name;
  }

  _log(lvl, message) {
    (async function () {
      const error = new Error();
      const stackTrace = error.stack;
      let func = "unknown";

      if (stackTrace) {
        const stackLines = stackTrace.split('\n');
        // stackLines.forEach(item => console.log(item));

        if (stackLines.length >= 5) {
          func = stackLines[4];
        }
      }

      await core.opAsync("op_log", lvl, func, message);
    })().catch((error) => {
      console.error("logger._log error:", error);
    });
  }

  debug(message) {
    const LVL = 0;
    this._log(LVL, message);
  }

  info(message) {
    const LVL = 5;
    this._log(LVL, message);
  }

  error(message) {
    const LVL = 9;
    this._log(LVL, message);
  }

  // async flush() {
  //   await core.opAsync("op_flush_log");
  // }
}

globalThis.log = new Logger('default')