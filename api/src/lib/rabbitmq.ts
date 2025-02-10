import { NewFileProcessQueue, OllamaModelPull } from "@/types/queue";
import client, { Connection, Channel, ConsumeMessage } from "amqplib";

const { RABBITMQ_URL } = process.env;

export enum Queue {
    NEW_FILE_EXTRACT = "NEW_FILE_EXTRACT",
    OLLAMA_MODEL_PULL = "OLLAMA_MODEL_PULL"
    // Add other queue names here
}


// Define interface for mapping Queue enum to message types
interface QueueMessageMap {
    [Queue.NEW_FILE_EXTRACT]: NewFileProcessQueue;
    [Queue.OLLAMA_MODEL_PULL]: OllamaModelPull;
}


class RabbitMQConnection {
    connection!: Connection;
    channel!: Channel;
    private connected!: boolean;

    async connect(max_tries = 3) {
        if (this.connected && this.channel) return this;
        else this.connected = false;
        for (let i = 0; i < max_tries; i++) {
            if (this.connected) break;
            try {
                this.connection = await client.connect(
                    RABBITMQ_URL || "amqp://localhost"
                );

                console.log(`✅ Rabbit MQ Connection is ready`);

                this.channel = await this.connection.createChannel();

                console.log(`🛸 Created RabbitMQ Channel successfully`);
                this.connected = true;
                return this;
            } catch (error) {
                console.error(`❌ Failed to connect to RabbitMQ Server. Retrying...`);
                console.error(error);
                this.connected = false;
            }
        }
    }

    // Type-safe sendToQueue method
    async sendToQueue<Q extends Queue>(
        queue: Q,
        message: QueueMessageMap[Q]
    ): Promise<boolean> {
        try {
            if (!this.channel) {
                await this.connect();
            }

            // Assert queue before sending message
            await this.channel.assertQueue(queue, {
                durable: true,  // Queue survives broker restart
            });

            return this.channel.sendToQueue(
                queue,
                Buffer.from(JSON.stringify(message)),
                {
                    persistent: true  // Message survives broker restart
                }
            );
        } catch (error) {
            console.error(error);
            throw error;
        }
    }

    // Type-safe consume method
    async consume<Q extends Queue>(
        queue: Q,
        callback: (message: QueueMessageMap[Q]) => Promise<void>
    ): Promise<void> {
        try {
            if (!this.channel) {
                await this.connect();
            }

            await this.channel.assertQueue(queue, { 
                durable: true,  // Queue survives broker restart
            });

            this.channel.consume(queue, async (msg) => {
                if (msg) {
                    const content = JSON.parse(msg.content.toString()) as QueueMessageMap[Q];
                    await callback(content);
                    this.channel.ack(msg);
                }
            });
        } catch (error) {
            console.error(error);
            throw error;
        }
    }
}

const mqConnection = new RabbitMQConnection();
export default mqConnection;