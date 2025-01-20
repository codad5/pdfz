import { Redis } from 'ioredis';
import { ProcessOptions } from '@/types/request';

const redis = new Redis(process.env.REDIS_URL || 'redis://localhost:6379');

async function isFileInProcess(fileId: string): Promise<boolean> {
    const exists = await redis.get(`processing:${fileId}`);
    return !!exists;
}

async function markFileAsProcessing(
    fileId: string, 
    options: ProcessOptions, 
    ttl: number = 3600
): Promise<void> {
    await redis.setex(
        `processing:${fileId}`, 
        ttl, 
        JSON.stringify({
            timestamp: new Date(),
            options
        })  
    );
}