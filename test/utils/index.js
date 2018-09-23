import { spawn } from 'child_process';

import {
    STORAGE_PATH,
    createStorage,
    dropStorage,
} from './storage';
import {
    TELEGRAM_API_URL,
    TELEGRAM_BOT_TOKEN,
    runServer,
    stopServer,
} from './telegram';

const APP_PATH = 'target/debug/comrade-colonel-bot';

/**
 * Seconds.
 */
export const DELETION_PERIOD = 1;
const IS_DEBUG = Boolean(process.env.DEBUG);
/**
 * Seconds.
 */
const MESSAGE_LIFETIME = 2;
let appProcess = null;

const onCloseApp = (code) => {
    throw new Error(`The app "${APP_PATH}" was closed with the code ${code}.`);
};

export const runApp = async (storageStub = null, options = {}) => {
    const { client } = await runServer();

    dropStorage();
    await createStorage(storageStub, client);

    expect(appProcess).toBeNull();
    const processOptions = {
        env: Object.assign({
            DELETION_PERIOD,
            MESSAGE_LIFETIME,
            RUST_BACKTRACE: '1',
            RUST_LOG: 'comrade_colonel_bot=debug',
            STORAGE_PATH,
            TELEGRAM_API_URL,
            TELEGRAM_BOT_TOKEN,
        }, options.env),
    };

    if (IS_DEBUG) {
        processOptions.stdio = 'inherit';
    }

    appProcess = spawn(APP_PATH, [], processOptions);
    appProcess.on('close', onCloseApp);
    return client;
};

export const stopApp = async () => {
    if (!IS_DEBUG) {
        dropStorage();
    }

    await stopServer();
    expect(appProcess).not.toBeNull();
    appProcess
        .off('close', onCloseApp)
        .kill();
    appProcess = null;
};

export const now = () => Math.floor(Date.now() / 1000);

/**
 * @param {number} duration - Seconds.
 */
export const delay = duration => new Promise(
    resolve => setTimeout(resolve, duration * 1000),
);
