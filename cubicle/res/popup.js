'use strict';

import {
  default as redirect,
  messageContainerSelection,
  updateContainerListing,
} from './components/context.js';

/**
 * Messages the background that an identity details update is requested,
 * then updates the popup.
 */
function messageContainerUpdate() {
  const selectContainer = document.getElementById('select-container');
  redirect({
    view: 'update_container',
    cookie_store_id: selectContainer.value,
  });
}

/**
 * Main entrypoint for popup creation, mainly for attaching listeners.
 * This is an IIFE as this is the first function to be executed.
 */
(function main() {
  const selectContainer = document.getElementById('select-container');
  selectContainer.addEventListener('change', (event) => {
    messageContainerSelection(event.target.value);
  });
  document.getElementById('btn-icon')
      .addEventListener('click', messageContainerUpdate);
  document.getElementById('btn-delete')
      .addEventListener('click', () => redirect({
        view: 'delete_prompt',
        cookie_store_id: selectContainer.value,
      }));
  document.getElementById('btn-options')
      .addEventListener('click', () => {
        window.open(browser.runtime.getURL('options.html'));
        window.close();
      });

  updateContainerListing().then(() => {
    messageContainerSelection(selectContainer.value);
  });
})();
