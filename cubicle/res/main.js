import init from './cubicle.js';

const listenerMap = new Map();
const wasmLoaded = init();

export function addRuntimeListener(event, handler) {
    listenerMap.set(event, handler);
}

for (const runtimeProperty in browser.runtime) {
    if (!runtimeProperty.startsWith('on')) continue;
    browser.runtime[runtimeProperty].addListener((...handlerArgs) => {
        wasmLoaded.then(async () => {
            if (listenerMap.has(runtimeProperty))
                await listenerMap.get(runtimeProperty)(handlerArgs);
        });
    });
}
