'use strict';

import {message_container_selection} from '../popup.js';

{% for name in view_names %}
    import context{{loop.index}} from './{{name}}.js';
{% endfor %}

const CONTEXT_MAP = new Map([
    {% for name in view_names %}
        ['{{name}}', context{{loop.index}}],
    {% endfor %}
]);

CONTEXT_MAP.set('update-container', CONTEXT_MAP.get('new-container'));

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

export function state_update_redirect(messageType, messageEnum) {
    const mainElement = document.getElementsByTagName('main')[0];
    mainElement.replaceChildren();
    const selectContainer = document.getElementById('select-container');
    selectContainer.disabled = true;
    const message = {message_type: messageType, action: messageEnum};
    return browser.runtime.sendMessage(message).then((html) => {
        selectContainer.innerHTML = html;
        selectContainer.disabled = false;
        message_container_selection(selectContainer.value);
    });
}
