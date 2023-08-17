'use strict';

import {stateUpdateRedirect} from './context.js';

/**
 * Changes the color of a text input element to correspond to the suffix type.
 * @param {HTMLInputElement} element - Text input element with an encoded
 *     suffix ID and raw suffix value.
 */
function colorizeSuffixInput(element) {
  switch (element.value.charAt(0)) {
    case '*': element.style.color = 'orange'; break;
    case '!': element.style.color = 'crimson'; break;
    default: element.style.color = 'black';
  }
}

/**
 * Messages the background that a suffix entry will need to be modified,
 * then updates the popup.
 * @param {HTMLInputElement} element - Text input element with an encoded
 *     suffix ID and raw suffix value.
 */
function messageUpdateSuffix(element) {
  const selectContainer = document.getElementById('select-container');
  stateUpdateRedirect('container_action', {
    action: 'update_suffix',
    cookie_store_id: selectContainer.value,
    old_suffix: element.id.slice('suffix-'.length),
    new_suffix: element.value,
  });
}

/**
 * Entrypoint for the container detail body.
 * Mainly for attaching listeners.
 */
export default function main() {
  for (const element of document.getElementsByClassName('input-suffix')) {
    element.addEventListener('input', (event) => {
      colorizeSuffixInput(event.target);
    });
    element.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') messageUpdateSuffix(event.target);
    });
  }
}
