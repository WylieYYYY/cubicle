'use strict';

/**
 * Messages the background that a PSL update is requested,
 * then updates the "last updated" date.
 * @param {Event} event - Generated click event,
 *     for disabling the button on click.
 */
function messagePslUpdate(event) {
  event.target.disabled = true;
  const lblPslDate = document.getElementById('lbl-psl-date');
  browser.runtime.sendMessage({message_type: 'psl_update', url: null})
      .then((newDate) => lblPslDate.innerText = newDate);
}

/**
 * Entrypoint for the extension preferences page.
 * Mainly for attaching listeners.
 */
export default function main() {
  document.getElementById('btn-psl-update')
      .addEventListener('click', messagePslUpdate);
}
