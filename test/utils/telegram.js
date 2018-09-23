import TelegramServer from 'telegram-test-api';

const HOST = '127.0.0.1';
const PORT = 9042;
export const TELEGRAM_API_URL = `http://${HOST}:${PORT}/`;
export const TELEGRAM_BOT_TOKEN = '123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11';

let server = null;
let client = null;

export const runServer = async () => {
    expect(server).toBeNull();
    expect(client).toBeNull();
    server = new TelegramServer({
        host: HOST,
        port: PORT,
    });
    await server.start();
    client = server.getClient(TELEGRAM_BOT_TOKEN);
    return { client, server };
};

export const stopServer = async () => {
    expect(server).not.toBeNull();
    expect(client).not.toBeNull();
    await server.stop();
    server = null;
    client = null;
};
