'use strict';

import redirect from './context.js';

/**
 * Currently a stub as the import functionality has not been implemented.
 * Entrypoint for the welcome page displayed when no container is selected.
 * Mainly for attaching listeners.
 */
export default function main() {
  document.getElementById('btn-import')
      .addEventListener('click', () => redirect({view: 'import'}));
}
