'use strict';

function message_psl_update(event) {
    event.target.disabled = true;
    const lblPslDate = document.getElementById('lbl-psl-date');
    browser.runtime.sendMessage({message_type: 'psl_update', url: null})
        .then((newDate) => lblPslDate.innerText = newDate);
}

export default function main() {
    document.getElementById('btn-psl-update')
        .addEventListener('click', message_psl_update);
}
