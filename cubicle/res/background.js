'use strict';

import {
  default as init, onMessage, onTabRemoved, onTabUpdated,
} from './cubicle.js';

const listenerMap = new Map();
const wasmLoaded = init();

/**
 * Adds a function as a listener of a runtime event.
 * @param {string} event - Name of the runtime event to listen for.
 * @param {Function} handler - Handler to be added as a listener of the event.
 */
export function addRuntimeListener(event, handler) {
  listenerMap.set(event, handler);
}

/**
 * Main entrypoint for initializing the background page,
 * uses promises to wait for WASM and attaches runtime listeners.
 * This is an IIFE as this is the first function to be executed.
 */
(function main() {
  for (const runtimeProperty in browser.runtime) {
    if (!runtimeProperty.startsWith('on')) continue;
    browser.runtime[runtimeProperty].addListener((...handlerArgs) => {
      return wasmLoaded.then(async () => {
        if (listenerMap.has(runtimeProperty)) {
          return listenerMap.get(runtimeProperty)(...handlerArgs);
        }
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
