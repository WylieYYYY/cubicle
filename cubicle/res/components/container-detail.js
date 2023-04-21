'use strict';

function colorize_suffix_input(event) {
    switch (event.target.value.charAt(0)) {
        case '*': event.target.style.color = 'orange';  break;
        case '!': event.target.style.color = 'crimson'; break;
        default:  event.target.style.color = 'black';
    }
}

export default function main() {
    for (const element of document.getElementsByClassName('input-suffix')) {
        element.addEventListener('input', colorize_suffix_input);
    }
}
