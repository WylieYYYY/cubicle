'use strict';

import {state_update_redirect} from './context.js';

function message_container_deletion() {
    const selectContainer = document.getElementById('select-container');
    state_update_redirect({
        message_type: 'delete_container',
        cookie_store_id: selectContainer.value
    });
}

export default function main() {
    document.getElementById('btn-yes')
        .addEventListener('click', message_container_deletion);
}
