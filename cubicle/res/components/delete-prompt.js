'use strict';

import {
  logStatus,
  messageContainerSelection,
  stateUpdateRedirect,
} from './context.js';

/**
 * Messages the background that a container deletion is requested,
 * then updates the popup.
 * @param {string} value - The ID of the selected container if it starts with
 *     [COOKIE_STORE_ID_MARKER_PREFIX], `new` if a new container is requested,
 *     and `none` if "no container" (default cookie store) is selected.
 * @return {Promise} Promise that fulfils once the deletion is fully complete.
 */
function messageContainerDeletion(value) {
  return stateUpdateRedirect('container_action', {
    action: {
      action: 'delete_container',
      cookie_store_id: value,
    },
  }).then(logStatus('Container was deleted'));
}

/**
 * Entry for the deletion prompt.
 * Mainly for attaching listeners.
 */
export default function main() {
  const selectContainer = document.getElementById('select-container');
  selectContainer.disabled = true;
  const enableSelection = () => selectContainer.disabled = false;

  document.getElementById('btn-yes').addEventListener('click', () => {
    messageContainerDeletion(selectContainer.value).then(enableSelection);
  });
  document.getElementById('btn-no').addEventListener('click', () => {
    messageContainerSelection(selectContainer.value).then(enableSelection);
  });
}
