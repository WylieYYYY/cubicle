'use strict';

function message_psl_update(event) {
    event.target.disabled = true;
}

export default function main() {
    document.getElementById('btn-psl-update')
        .addEventListener('click', message_psl_update);
}
