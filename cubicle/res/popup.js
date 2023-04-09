'use strict';

import context_new_container from './components/new-container.js';

function colorize_suffix_input(event) {
    switch (event.target.value.charAt(0)) {
        case '*': event.target.style.color = 'orange';  break;
        case '!': event.target.style.color = 'crimson'; break;
        default:  event.target.style.color = 'black';
    }
}

function message_container_selection(event) {
    const mainElement = document.getElementsByTagName('main')[0];
    mainElement.replaceChildren();
    if (event.target.value === 'new') {
        browser.runtime.sendMessage({message_type: 'request_new_container'})
            .then(html => {
                mainElement.innerHTML = html;
                context_new_container();
            });
    }
}

(function main() {
    document.getElementById('select-container')
        .addEventListener('change', message_container_selection);
    for (const element of document.getElementsByClassName('input-suffix')) {
        element.addEventListener('input', colorize_suffix_input);
    }
})();