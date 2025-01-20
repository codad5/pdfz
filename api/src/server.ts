import express, { Express, Request, Response, Application } from 'express';
import dotenv from 'dotenv';
import { upload, uploadExists } from '@/helpers/uploadhelper';
import { ResponseHelper } from '@/helpers/response';
import mqConnection, { Queue } from '@/lib/rabbitmq';
import { NewFileProcessQueue } from '@/types/queue';
import { ProcessResponse, UploadResponse } from '@/types/response';
import { ProcessOptions } from '@/types/request';
import { isFileInProcess, markFileAsProcessing } from '@/lib/redis';

dotenv.config();

const app: Application = express();
const port = process.env.API_PORT || 3000;

// Initialize connections before starting server
async function initializeConnections() {
    try {
        await mqConnection.connect();
        console.log('Connected to RabbitMQ');
        return true;
    } catch (error) {
        console.error('Failed to connect to RabbitMQ:', error);
        return false;
    }
}

// Middleware
app.use(express.json());
app.use((req, res, next) => {
    ResponseHelper.registerExpressResponse(req, res);
    next();
});

// Routes
app.get('/', (req: Request, res: Response) => {
    res.send('Welcome to Express & TypeScript Server');
});

app.post('/upload', upload.single('pdf'), async (req: Request, res: Response) => {
    try {
        if (!req.file) {
            throw new Error('File is missing');
        }
        ResponseHelper.success<UploadResponse>({
            id: req.file.filename.split('.').slice(0, -1).join('.'),
            filename: req.file.filename,
            path: req.file.path,
            size: req.file.size
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'File upload failed',
            { message: (error as Error).message ?? 'File upload failed' }
        );
    }
});

app.post('/process/:id', async (req: Request, res: Response) => {
    try {
        const { id } = req.params;
        const params = req.body as ProcessOptions;
        const startPage = params?.startPage ?? 1;
        const pageCount = params?.pageCount ?? 0;
        const priority = params?.priority ?? 1;

        if (!uploadExists(`${id}.pdf`)) {
            throw new Error('File not found');
        }

        if (await isFileInProcess(id)) {
            throw new Error('File is already in process');
        }

        const d = await mqConnection.sendToQueue(Queue.NEW_FILE_EXTRACT, {
            file: `${id}.pdf`,
            start_page: startPage,
            page_count: pageCount
        });

        if (!d) {
            throw new Error('Failed to send file to queue');
        }

        await markFileAsProcessing(id);

        ResponseHelper.success<ProcessResponse>({
            id,
            file: `${id}.pdf`,
            message: 'File processing started',
            options: { startPage, pageCount, priority },
            status: 'queued',
            queuedAt: new Date()
        });
    } catch (error) {
        ResponseHelper.error(
            (error as Error).message ?? 'File processing failed',
            { message: (error as Error).message ?? 'File processing failed' }
        );
    }
});

// Start server only after establishing connections
async function startServer() {
    const isConnected = await initializeConnections();
    
    if (isConnected) {
        app.listen(port, () => {
            console.log(`Server is Fire at http://localhost:${port}`);
        });
    } else {
        console.error('Failed to initialize required connections. Exiting...');
        process.exit(1);
    }
}

startServer();

