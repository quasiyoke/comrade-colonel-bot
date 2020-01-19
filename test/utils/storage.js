import fs from 'fs';

import SQL from 'sql-template-strings';
import sqlite from 'sqlite';

import { delay } from './misc';

/**
 * Seconds.
 */
const LAST_ACTION_DELAY = 0.1;
export const STORAGE_PATH = './test-db.sqlite3';
let client = null;

/**
 * Checks messages remaining at fake Telegram server.
 */
const assertServerMessages = async (expectedMessages) => {
    await delay(LAST_ACTION_DELAY);
    const actualMessages = await client.getUpdatesHistory();
    const actualMessageIds = actualMessages.map(update => update.messageId);
    const expectedMessageIds = expectedMessages.map(update => update.telegram_id);
    expect(actualMessageIds).toEqual(expectedMessageIds, 'Messages in API server should match specified messages');
};

/**
 * Checks messages remaining at SQLite storage.
 */
const assertStorageMessages = async (expectedMessages) => {
    await delay(LAST_ACTION_DELAY);
    const db = await sqlite.open(STORAGE_PATH);
    const actualMessages = await db.all(
        SQL`SELECT
            id,
            telegram_id,
            chat_telegram_id,
            date
        FROM message;`,
    );
    expect(actualMessages).toEqual(expectedMessages, 'Messages in DB should match specified messages');
};

export const assertStorage = ({
    messages,
    serverMessages = messages,
    storageMessages = messages,
}) => Promise.all([
    assertServerMessages(serverMessages),
    assertStorageMessages(storageMessages),
]);

const fillStorage = async ({ messages = [] }) => {
    const db = await sqlite.open(STORAGE_PATH);
    await db.run(
        SQL`CREATE TABLE message (
            id                  INTEGER PRIMARY KEY,
            telegram_id         INTEGER NOT NULL,
            chat_telegram_id    INTEGER NOT NULL,
            date                INTEGER NOT NULL
        );`,
    );

    for (const { telegram_id, chat_telegram_id, date } of messages) {
        await db.run(
            SQL`INSERT INTO message (telegram_id, chat_telegram_id, date)
                VALUES (${telegram_id}, ${chat_telegram_id}, ${date})
            ;`,
        );
    }
};

export const createStorage = async (storageStub, newClient) => {
    if (storageStub) {
        await fillStorage(storageStub);
    }

    expect(client).toBeNull();
    client = newClient;
};

export const dropStorage = () => {
    try {
        fs.unlinkSync(STORAGE_PATH);
        // eslint-disable-next-line no-empty
    } catch (err) {
    }

    client = null;
};
