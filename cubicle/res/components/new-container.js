'use strict';

function message_submit_identity_details(event) {
    const identityDetails = {};
    for (const [key, value] of new FormData(event.target).entries()) identityDetails[key] = value;
    browser.runtime.sendMessage({
        message_type: 'submit_identity_details',
        cookie_store_id: null, details: identityDetails
    });
}

export default function main() {
    document.getElementById('form-new-container')
        .addEventListener('submit', message_submit_identity_details);
}