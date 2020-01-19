export const now = () => Math.floor(Date.now() / 1000);

/**
 * @param {number} duration - Seconds.
 */
export const delay = duration => new Promise(
    resolve => setTimeout(resolve, duration * 1000),
);
