'use strict';

function colorize_suffix_input(event) {
    switch (event.target.value.charAt(0)) {
        case '*': event.target.style.color = 'orange';  break;
        case '!': event.target.style.color = 'crimson'; break;
        default:  event.target.style.color = 'black';
    }
}

function message_container_selection(event) {
    if (event.target.value === 'new') {
        browser.runtime.sendMessage({details: {
            color: 'blue', icon: 'circle', name: 'Cubicle'
        }});
    }
}

(function main() {
    document.getElementById('select-container')
        .addEventListener('change', message_container_selection);
    for (const element of document.getElementsByClassName('input-suffix')) {
        element.addEventListener('input', colorize_suffix_input);
    }
})();