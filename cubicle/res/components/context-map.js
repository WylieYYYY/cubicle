'use strict';

// This file will be templated at build time
// to import all view listener attaching functions.

{% for name in view_names %}
    import context{{loop.index}} from './{{name}}.js';
{% endfor %}

export const CONTEXT_MAP = new Map([
    {% for name in view_names %}
        ['{{name}}', context{{loop.index}}],
    {% endfor %}
]);
