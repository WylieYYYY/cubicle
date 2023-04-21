'use strict';

{% for name in view_names %}
    import context{{loop.index}} from './{{name}}.js';
{% endfor %}

const CONTEXT_MAP = new Map([
    {% for name in view_names %}
        ['{{name}}', context{{loop.index}}],
    {% endfor %}
]);

export const COOKIE_STORE_ID_MARKER_PREFIX = "b64_";

export default function redirect(viewEnum) {
    const mainElement = document.getElementsByTagName('main')[0];
    mainElement.replaceChildren();
    return browser.runtime.sendMessage({
        message_type: 'request_page', view: viewEnum
    }).then((html) => {
        mainElement.innerHTML = html;
        CONTEXT_MAP.get(viewEnum.view.replaceAll('_', '-'))(viewEnum);
    });
}
