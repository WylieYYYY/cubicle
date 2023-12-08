'use strict';

import {
  COOKIE_STORE_ID_MARKER_PREFIX,
  stateUpdateRedirect,
} from './context.js';

/**
 * Messages the background that some container details have changed,
 * then updates the popup.
 * @param {Event} event - Generated submit event, for extracting form data.
 */
function messageSubmitIdentityDetails(event) {
  const selectValue = document.getElementById('select-container').value;
  const cookieStoreId = selectValue.startsWith(
      COOKIE_STORE_ID_MARKER_PREFIX)? selectValue : null;

  const identityDetails = {};
  for (const [key, value] of new FormData(event.target).entries()) {
    identityDetails[key] = value;
  }
  stateUpdateRedirect('container_action', {
    action: {
      action: 'submit_identity_details',
      cookie_store_id: cookieStoreId, details: identityDetails,
    },
  });
}

/**
 * Entrypoint for the new / update container menu.
 * This is dual use and the name may be changed in the future for clarity.
 * Mainly for attaching listeners.
 */
export default function main() {
  document.getElementById('form-new-container')
      .addEventListener('submit', messageSubmitIdentityDetails);
}
