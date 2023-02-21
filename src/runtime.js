((globalThis) => {
    const core = Deno.core;

    globalThis.console = {
        log: (...args) => {
            core.print(`[log]: ${argsToMessage(...args)}\n`, false);
        },
        error: (...args) => {
            core.print(`[err]: ${argsToMessage(...args)}\n`, true);
        },
    };


    function argsToMessage(...args) {
        return args.map((arg) => JSON.stringify(arg)).join(" ");
    }

})(globalThis);