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
 * Messages the background that the preferences should be saved and applied.
 * @param {Event} event - Generated submit event, for extracting form data.
 */
function messageApplyPreferences(event) {
  const preferences = {};
  for (const [key, value] of new FormData(event.target).entries()) {
    preferences[key] = value;
  }
  browser.runtime.sendMessage({
    message_type: 'apply_preferences',
    preferences: preferences,
  });
}

/**
 * Entrypoint for the extension preferences page.
 * Mainly for attaching listeners.
 */
export default function main() {
  document.getElementById('btn-psl-update')
      .addEventListener('click', messagePslUpdate);
  document.getElementById('form-preferences')
      .addEventListener('submit', messageApplyPreferences);
}
