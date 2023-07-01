'use strict';
import {
    default as init, onMessage, onTabRemoved, onTabUpdated
} from './cubicle.js';

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

    browser.tabs.onRemoved.addListener((tabId) => {
        wasmLoaded.then(async () => onTabRemoved(tabId));
    });
    browser.tabs.onUpdated.addListener((tabId, _changeInfo, tab) => {
        wasmLoaded.then(async () => onTabUpdated(tabId, tab));
    }, {properties: ['url']});
})();
