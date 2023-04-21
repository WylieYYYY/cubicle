'use strict';

import {COOKIE_STORE_ID_MARKER_PREFIX} from './context.js';

function message_submit_identity_details(event) {
    const selectValue = document.getElementById('select-container').value;
    const cookieStoreId = selectValue.startsWith(
        COOKIE_STORE_ID_MARKER_PREFIX)? selectValue : null;

    const identityDetails = {};
    for (const [key, value] of new FormData(event.target).entries()) identityDetails[key] = value;
    browser.runtime.sendMessage({
        message_type: 'submit_identity_details',
        cookie_store_id: cookieStoreId, details: identityDetails
    });
}

export default function main() {
    document.getElementById('form-new-container')
        .addEventListener('submit', message_submit_identity_details);
}
