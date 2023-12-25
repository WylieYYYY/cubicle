'use strict';

import {
  logStatus,
  messageContainerSelection,
  stateUpdateRedirect,
} from './context.js';

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
 * Messages the background that the recorded suffixes are acceptable,
 * and should be persisted as a permanent container.
 */
function messageConfirmRecording() {
  const selectContainer = document.getElementById('select-container');
  stateUpdateRedirect('container_action', {
    action: {
      action: 'confirm_recording',
      cookie_store_id: selectContainer.value,
    },
  }).then(logStatus('Recoding confirmed'));
}

/**
 * Messages the background that a suffix entry will need to be modified,
 * then updates the popup.
 * @param {string} encodedOldSuffix - Encoded version of the old suffix, can be
 *     extracted from assosciated element's ID.
 * @param {string} newSuffix - New suffix for replacement, empty string for
 *     deleting the suffix instead.
 */
function messageUpdateSuffix(encodedOldSuffix, newSuffix) {
  const selectContainer = document.getElementById('select-container');
  stateUpdateRedirect('container_action', {
    action: {
      action: 'update_suffix',
      cookie_store_id: selectContainer.value,
      old_suffix: encodedOldSuffix,
      new_suffix: newSuffix,
    },
  }).then(logStatus(`Suffix '${newSuffix}' was added`));
}

/**
 * Entrypoint for the container detail body.
 * Mainly for attaching listeners.
 */
export default function main() {
  document.getElementById('btn-refresh')?.addEventListener('click', () => {
    const selectContainer = document.getElementById('select-container');
    messageContainerSelection(selectContainer.value);
  });
  document.getElementById('btn-confirm-recording')?.addEventListener('click',
      messageConfirmRecording);

  for (const element of document.getElementsByClassName('input-suffix')) {
    const encodedOldSuffix = element.id.slice('suffix-'.length);

    element.addEventListener('input', (event) => {
      colorizeSuffixInput(event.target);
    });
    element.addEventListener('keydown', (event) => {
      if (event.key === 'Enter') {
        messageUpdateSuffix(encodedOldSuffix, event.target.value);
      }
    });

    document.getElementById('btn-option-' + encodedOldSuffix)
        .addEventListener('click', () => {
          messageUpdateSuffix(encodedOldSuffix, '');
        });
  }
}
