'use strict';

import {
    default as redirect,
    COOKIE_STORE_ID_MARKER_PREFIX
} from './components/context.js';

function message_container_selection(event) {
    if (event.target.value === 'new') redirect({view: 'new_container'});
    else if (event.target.value === 'none') redirect({view: 'welcome'});

    const btnDelete = document.getElementById('btn-delete');
    if (event.target.value.startsWith(COOKIE_STORE_ID_MARKER_PREFIX)) {
        btnDelete.style.visibility = 'visible';
    } else btnDelete.style.visibility = 'hidden';
}

function message_container_update() {
    const selectContainer = document.getElementById('select-container');
    redirect({
        view: 'update_container',
        cookie_store_id: selectContainer.value
    });
}

(function main() {
    document.getElementById('select-container')
        .addEventListener('change', message_container_selection);
    document.getElementById('btn-icon')
        .addEventListener('click', message_container_update);
    redirect({view: 'welcome'});
    browser.runtime.sendMessage({
        message_type: 'request_page', view: {view: 'fetch_all_containers'}
    }).then((html) => {
        const selectElement = document.getElementById('select-container');
        selectElement.innerHTML = html;
    });
})();
