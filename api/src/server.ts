import express, { Express, Request, Response , Application } from 'express';
import dotenv from 'dotenv';
import multer from 'multer';
import path from 'path';
import fs from 'fs';
import { upload } from '@/helpers/uploadhelper';

//For env File 
dotenv.config();

const app: Application = express();
const port = process.env.API_PORT || 3000;



app.get('/', (req: Request, res: Response) => {
  res.send('Welcome to Express & TypeScript Server');
});

app.post('/upload', upload.single('pdf'), (req: Request, res: Response) => {
  try {
    if (!req.file) throw new Error('File is missing');
    res.status(200).json({
      message: 'File uploaded successfully',
      file: req.file,
    });
  } catch (error) {
    res.status(400).json({ message: (error as Error).message ?? 'File upload failed' });
  }
});

app.listen(port, () => {
  console.log(`Server is Fire at http://localhost:${port}`);
});