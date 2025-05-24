'use strict';

import {
  default as init, onMessage, onTabRemoved, onTabUpdated,
} from './cubicle.js';

const wasmLoaded = init();

/**
 * Main entrypoint for initializing the background page,
 * uses promises to wait for WASM and attaches runtime listeners.
 * This is an IIFE as this is the first function to be executed.
 */
(function main() {
  browser.runtime.onMessage.addListener((message) => {
    return wasmLoaded.then(async () => onMessage(message));
  });

  browser.tabs.onRemoved.addListener((tabId) => {
    wasmLoaded.then(async () => onTabRemoved(tabId));
  });
  browser.tabs.onUpdated.addListener((tabId, _changeInfo, tab) => {
    wasmLoaded.then(async () => onTabUpdated(tabId, tab));
  }, {properties: ['url']});
})();
