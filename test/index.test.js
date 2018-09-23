import {
    DELETION_PERIOD,
    delay,
    now,
    runApp,
    stopApp,
} from './utils';
import { assertStorage } from './utils/storage';

describe('Regular chat message', () => {
    let date;

    const testDeletionAfterMessageLifetime = (lifetime) => {
        describe(`when message lifetime is ${lifetime} s`, () => {
            let client;
            const ageYoung = lifetime - DELETION_PERIOD;
            const ageOld = lifetime;

            beforeEach(async () => {
                client = await runApp(null, {
                    env: {
                        MESSAGE_LIFETIME: lifetime,
                    },
                });
            });

            describe(`and it's too young (${ageYoung} s old)`, () => {
                beforeEach(async () => {
                    const message = client.makeMessage('Some regular chat message', {
                        chat: {
                            id: 100,
                        },
                        date: date - ageYoung,
                    });
                    await client.sendMessage(message);
                });

                test('isn\'t deleted', async () => {
                    await delay(DELETION_PERIOD);
                    await assertStorage({
                        messages: [
                            {
                                id: 1,
                                telegram_id: 1,
                                chat_telegram_id: 100,
                                date: date - ageYoung,
                            },
                        ],
                    });
                });
            });

            describe(`and it's too old (${ageOld} s old)`, () => {
                beforeEach(async () => {
                    const message = client.makeMessage('Some regular chat message', {
                        chat: {
                            id: 100,
                        },
                        date: date - ageOld,
                    });
                    await client.sendMessage(message);
                });

                test('is deleted', async () => {
                    await delay(DELETION_PERIOD);
                    await assertStorage({
                        messages: [],
                    });
                });
            });
        });
    };

    beforeEach(() => {
        date = now();
    });

    afterEach(stopApp);

    describe('when there\'s no DB', () => {
        beforeEach(async () => {
            const client = await runApp();
            const message = client.makeMessage('Some regular chat message', {
                chat: {
                    id: 200,
                },
                date,
            });
            await client.sendMessage(message);
        });

        test('is written to the fresh DB', async () => {
            await assertStorage({
                messages: [
                    {
                        id: 1,
                        telegram_id: 1,
                        chat_telegram_id: 200,
                        date,
                    },
                ],
            });
        });
    });

    describe('when there\'s an existing DB', () => {
        beforeEach(async () => {
            const client = await runApp({
                messages: [
                    {
                        telegram_id: 100,
                        chat_telegram_id: 200,
                        date,
                    },
                    {
                        telegram_id: 101,
                        chat_telegram_id: 200,
                        date,
                    },
                ],
            });
            const message = client.makeMessage('Some regular chat message', {
                chat: {
                    id: 201,
                },
                date,
            });
            await client.sendMessage(message);
        });

        test('is appended to the DB', async () => {
            await assertStorage({
                serverMessages: [
                    {
                        id: 3,
                        telegram_id: 1,
                        chat_telegram_id: 201,
                        date,
                    },
                ],
                storageMessages: [
                    {
                        id: 1,
                        telegram_id: 100,
                        chat_telegram_id: 200,
                        date,
                    },
                    {
                        id: 2,
                        telegram_id: 101,
                        chat_telegram_id: 200,
                        date,
                    },
                    {
                        id: 3,
                        telegram_id: 1,
                        chat_telegram_id: 201,
                        date,
                    },
                ],
            });
        });
    });

    testDeletionAfterMessageLifetime(30);
    testDeletionAfterMessageLifetime(42 * 60 * 60);
});
