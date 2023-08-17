'use strict';

import {
    default as redirect,
    COOKIE_STORE_ID_MARKER_PREFIX
} from './components/context.js';

export function message_container_selection(value) {
    if (value === 'new') redirect({view: 'new_container'});
    else if (value === 'none') redirect({view: 'welcome'});
    else redirect({
        view: 'container_detail', cookie_store_id: value
    });

    const btnDelete = document.getElementById('btn-delete');
    if (value.startsWith(COOKIE_STORE_ID_MARKER_PREFIX)) {
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
    const selectContainer = document.getElementById('select-container');
    selectContainer.addEventListener('change', (event) => {
        message_container_selection(event.target.value);
    });
    document.getElementById('btn-icon')
        .addEventListener('click', message_container_update);
    document.getElementById('btn-delete')
        .addEventListener('click', () => redirect({
            view: 'delete_prompt',
            cookie_store_id: selectContainer.value
        }));
    document.getElementById('btn-options')
        .addEventListener('click', () => {
            window.open(browser.runtime.getURL('options.html'));
            window.close();
        });
    redirect({view: 'welcome'});
    browser.runtime.sendMessage({
        message_type: 'request_page', view: {
            view: 'fetch_all_containers', selected: null
        }
    }).then((html) => {
        const selectElement = document.getElementById('select-container');
        selectElement.innerHTML = html;
    });
})();
