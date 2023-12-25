'use strict';

import {
  COOKIE_STORE_ID_MARKER_PREFIX,
  logStatus,
  stateUpdateRedirect,
} from './context.js';

/**
 * Messages the background that some container details have changed,
 * then updates the popup.
 * @param {Event} event - Generated submit event, for extracting form data.
 */
function messageSubmitIdentityDetails(event) {
  const selectValue = document.getElementById('select-container').value;
  const shouldRecord = event.submitter.id === 'btn-recording';
  const cookieStoreId = selectValue.startsWith(
      COOKIE_STORE_ID_MARKER_PREFIX)? selectValue : null;

  const identityDetails = {};
  for (const [key, value] of new FormData(event.target).entries()) {
    identityDetails[key] = value;
  }

  const verb = cookieStoreId === null? 'created' : 'updated';
  stateUpdateRedirect('container_action', {
    action: {
      action: 'submit_identity_details',
      cookie_store_id: cookieStoreId, details: identityDetails,
      should_record: shouldRecord,
    },
  }).then(logStatus('Container was ' + verb));
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
