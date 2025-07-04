// commitlint.config.mjs
export default {
    extends: ['@commitlint/config-conventional'],
    rules: {
        'body-max-line-length': [2, 'always', 250], //Override the default body line length to 250 characters
    },
};
