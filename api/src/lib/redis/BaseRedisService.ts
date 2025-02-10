// src/lib/redis/BaseRedisService.ts
import { Redis } from 'ioredis';

const redis = new Redis(process.env.REDIS_URL || 'redis://localhost:6379');
export default redis;
export abstract class BaseRedisService {
    protected redis: Redis;
    protected prefix: string;

    constructor(prefix: string) {
        this.redis = redis;
        this.prefix = prefix;
    }

    protected async setStatus(id: string, status: string): Promise<void> {
        await this.redis.set(`${this.prefix}:status:${id}`, status);
    }

    protected async getStatus(id: string): Promise<string | null> {
        return await this.redis.get(`${this.prefix}:status:${id}`);
    }

    protected async setProgress(id: string, progress: number): Promise<void> {
        await this.redis.set(`${this.prefix}:progress:${id}`, progress.toString());
    }

    protected async getProgress(id: string): Promise<number> {
        const progress = await this.redis.get(`${this.prefix}:progress:${id}`);
        return progress ? parseInt(progress, 10) : 0;
    }

    protected async setWithTTL(key: string, value: string, ttl: number): Promise<void> {
        await this.redis.setex(key, ttl, value);
    }
}