'use strict';

{% for name in view_names %}
    import context{{loop.index}} from './{{name}}.js';
{% endfor %}

const CONTEXT_MAP = new Map([
    {% for name in view_names %}
        ['{{name}}', context{{loop.index}}],
    {% endfor %}
]);

export default function redirect(viewEnum) {
    const mainElement = document.getElementsByTagName('main')[0];
    mainElement.replaceChildren();
    return browser.runtime.sendMessage({
        message_type: 'request_page', view: viewEnum
    }).then((html) => {
        mainElement.innerHTML = html;
        CONTEXT_MAP.get(viewEnum.replaceAll('_', '-'))(viewEnum);
    });
}
