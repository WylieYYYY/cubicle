'use strict';

import {stateUpdateRedirect} from './context.js';

/**
 * Messages the background that a container migration is requested,
 * then updates the popup.
 * @param {object} migrateType - Specifies container provider with
 *     additional import details.
 */
function messageMigration(migrateType) {
  stateUpdateRedirect('migrate_container', {
    migrate_type: migrateType,
    detect_temp: document.getElementById('check-detect-temp').checked,
  });
}

/**
 * Entry for the import page.
 * Mainly for attaching listeners.
 */
export default function main() {
  document.getElementById('btn-native')
      .addEventListener('click', () => messageMigration({
        migrate_type: 'native',
      }));
}
