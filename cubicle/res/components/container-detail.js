'use strict';

import {state_update_redirect} from './context.js';

function colorize_suffix_input(event) {
    switch (event.target.value.charAt(0)) {
        case '*': event.target.style.color = 'orange';  break;
        case '!': event.target.style.color = 'crimson'; break;
        default:  event.target.style.color = 'black';
    }
}

function message_update_suffix(element) {
    const selectContainer = document.getElementById('select-container');
    state_update_redirect('container_action', {
        action: 'update_suffix',
        cookie_store_id: selectContainer.value,
        old_suffix: element.id.slice('suffix-'.length),
        new_suffix: element.value
    });
}

export default function main() {
    for (const element of document.getElementsByClassName('input-suffix')) {
        element.addEventListener('input', colorize_suffix_input);
        element.addEventListener('keydown', (event) => {
            if (event.key === 'Enter') message_update_suffix(event.target);
        });
    }
}
