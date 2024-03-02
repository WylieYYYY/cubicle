'use strict';

import {CONTEXT_MAP} from './context-map.js';

// update screen is the same as the new container screen and can be reused
CONTEXT_MAP.set('update-container', CONTEXT_MAP.get('new-container'));

/**
 * In-band marker prefix to denote that the following data is a valid
 * cookie store ID, otherwise it may be interpreted as a control sequence.
 */
export const COOKIE_STORE_ID_MARKER_PREFIX = 'b64_';

/**
 * Creates a function that displays a message in the status bar when called.
 * Useful for logging in a thenable handler.
 * @param {string} message - Message to be displayed.
 * @return {Function} Function that displays the message when called.
 */
export function logStatus(message) {
  return () => {
    document.getElementById('lbl-status').innerText = message;
  };
}

/**
 * Messages the background about a container selection, then updates the popup.
 * @param {string} value - The ID of the selected container if it starts with
 *     [COOKIE_STORE_ID_MARKER_PREFIX], `new` if a new container is requested,
 *     and `none` if "no container" (default cookie store) is selected.
 * @return {Promise} Promise that fulfils once the update is fully complete.
 */
export function messageContainerSelection(value) {
  const iconBtn = document.getElementById('btn-icon');
  const iconImg = document.getElementById('img-icon');

  const resetIconStyle = () => {
    iconBtn.style.visibility = 'hidden';
  };

  const btnDelete = document.getElementById('btn-delete');
  if (value.startsWith(COOKIE_STORE_ID_MARKER_PREFIX)) {
    btnDelete.style.visibility = 'visible';
  } else btnDelete.style.visibility = 'hidden';

  if (value === 'new') {
    return redirect({view: 'new_container'}).then(resetIconStyle);
  } else if (value === 'none') {
    return redirect({view: 'welcome'}).then(resetIconStyle);
  } else {
    return redirect({
      view: 'container_detail', cookie_store_id: value,
    }).then(() => {
      iconBtn.style.visibility = 'visible';
      iconBtn.style.backgroundColor = document
          .getElementById('data-icon-color').getAttribute('data-icon-color');
      iconImg.src = document.getElementById('data-icon-link')
          .getAttribute('data-icon-link');
    });
  }
}

/**
 * Updates the popup with the specified composed view.
 * This may be merged with [stateUpdateRedirect] in the future.
 * @param {object} viewEnum - View specification.
 * @return {Promise} Promise that fulfils once the view is rendered.
 */
export default async function redirect(viewEnum) {
  const mainElement = document.getElementsByTagName('main')[0];
  mainElement.replaceChildren();
  return browser.runtime.sendMessage({
    message_type: 'request_page', view: viewEnum,
  }).then((html) => {
    mainElement.innerHTML = html;
    CONTEXT_MAP.get(viewEnum.view.replaceAll('_', '-'))(viewEnum);
  });
}

/**
 * Sends a message to the background, and updates elements in the popup.
 * @param {string} messageType - Action type for determining
 *     elements to update.
 * @param {object} messageEnum - The actual message.
 * @return {Promise} Promise that fulfils once the update is fully complete.
 */
export async function stateUpdateRedirect(messageType, messageEnum) {
  const mainElement = document.getElementsByTagName('main')[0];
  mainElement.style.display = 'none';
  const selectContainer = document.getElementById('select-container');
  selectContainer.disabled = true;
  const message = {message_type: messageType, ...messageEnum};
  return browser.runtime.sendMessage(message).then((html) => {
    selectContainer.innerHTML = html;
    messageContainerSelection(selectContainer.value);
  }).finally(() => {
    selectContainer.disabled = false;
    mainElement.style.display = 'flex';
  });
}

/**
 * Updates the container listing in the `select-container` element.
 * @return {Promise} Promise that fulfils once the listing is updated.
 */
export async function updateContainerListing() {
  return browser.runtime.sendMessage({
    message_type: 'request_page', view: {
      view: 'fetch_all_containers', selected: null,
    },
  }).then((html) => {
    const selectElement = document.getElementById('select-container');
    selectElement.innerHTML = html;
  });
}
