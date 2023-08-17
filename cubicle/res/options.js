'use strict';

import redirect from './components/context.js';

/**
 * Main entrypoint for the preferences page.
 * This is an IIFE as this is a standalone page.
 */
(function main() {
  redirect({view: 'options_body'});
})();
