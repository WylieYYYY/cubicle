'use strict';

import {stateUpdateRedirect} from './context.js';

/**
 * Messages the background that a container deletion is requested,
 * then updates the popup.
 */
function messageContainerDeletion() {
  const selectContainer = document.getElementById('select-container');
  stateUpdateRedirect('container_action', {
    action: 'delete_container',
    cookie_store_id: selectContainer.value,
  });
}

/**
 * Entry for the deletion prompt.
 * Mainly for attaching listeners.
 */
export default function main() {
  document.getElementById('btn-yes')
      .addEventListener('click', messageContainerDeletion);
}
