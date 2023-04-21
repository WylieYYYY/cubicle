'use strict';
import {default as init, onMessage} from './cubicle.js';

const listenerMap = new Map();
const wasmLoaded = init();

export function addRuntimeListener(event, handler) {
    listenerMap.set(event, handler);
}

(function main() {
    for (const runtimeProperty in browser.runtime) {
        if (!runtimeProperty.startsWith('on')) continue;
        browser.runtime[runtimeProperty].addListener((...handlerArgs) => {
            return wasmLoaded.then(async () => {
                if (listenerMap.has(runtimeProperty))
                    return listenerMap.get(runtimeProperty)(...handlerArgs);
            });
        });
    }

    addRuntimeListener('onMessage', onMessage);
})();
