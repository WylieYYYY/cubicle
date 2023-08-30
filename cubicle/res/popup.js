'use strict';

import {
  default as redirect,
  COOKIE_STORE_ID_MARKER_PREFIX,
  updateContainerListing,
} from './components/context.js';

/**
 * Messages the background about a container selection, then updates the popup.
 * @param {string} value - The ID of the selected container if it starts with
 *     [COOKIE_STORE_ID_MARKER_PREFIX], `new` if a new container is requested,
 *     and `none` if "no container" (default cookie store) is selected.
 */
export function messageContainerSelection(value) {
  if (value === 'new') redirect({view: 'new_container'});
  else if (value === 'none') redirect({view: 'welcome'});
  else {
    redirect({
      view: 'container_detail', cookie_store_id: value,
    });
  }

  const btnDelete = document.getElementById('btn-delete');
  if (value.startsWith(COOKIE_STORE_ID_MARKER_PREFIX)) {
    btnDelete.style.visibility = 'visible';
  } else btnDelete.style.visibility = 'hidden';
}

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
  redirect({view: 'welcome'});
  updateContainerListing();
})();
