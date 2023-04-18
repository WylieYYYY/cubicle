'use strict';

import redirect from './components/context.js';

function colorize_suffix_input(event) {
    switch (event.target.value.charAt(0)) {
        case '*': event.target.style.color = 'orange';  break;
        case '!': event.target.style.color = 'crimson'; break;
        default:  event.target.style.color = 'black';
    }
}

function message_container_selection(event) {
    if (event.target.value === 'new') redirect('new_container');
}

(function main() {
    document.getElementById('select-container')
        .addEventListener('change', message_container_selection);
    for (const element of document.getElementsByClassName('input-suffix')) {
        element.addEventListener('input', colorize_suffix_input);
    }
    redirect('welcome');
    browser.runtime.sendMessage({
        message_type: 'request_page', view: 'fetch_all_containers'
    }).then((html) => {
        const selectElement = document.getElementById('select-container');
        selectElement.innerHTML = html;
    });
})();
