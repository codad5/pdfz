# PDFz

Developed by [codad5](https://github.com/codad5)

PDFz is designed to streamline the extraction and processing of text from PDF files, making it easier to manage and analyze large volumes of documents. By leveraging Rust for the Extractor Service, the project addresses performance bottlenecks, ensuring efficient and fast processing of PDF files.

This project consists of a microservices-based system for extracting and processing PDF files. It includes:

- **Extractor Service** (Rust): Handles file processing using Tesseract OCR.
- **API Service** (Node.js): Provides endpoints for uploading and managing file extraction.
- **Redis**: Caching and tracking progress.
- **RabbitMQ**: Message queuing between API and Extractor.
- **Docker**: Containerized deployment for all services.

---

## Features

- Upload PDF files via the API.
- Queue files for processing.
- Extract text from PDFs using OCR (Tesseract).
- Track file processing progress.
- Store extracted data.

---

## Upcoming Features

### Ollama Integration (Coming Soon)
We are working on integrating **Ollama** to enable advanced text processing capabilities using locally run large language models (LLMs). This will allow you to:
- Summarize extracted text from PDFs.
- Perform question-answering on the content.
- Generate insights or reports from the processed data.

Stay tuned for updates!

---

## Architecture

- **API Service**: Handles file uploads and processing requests.
- **Extractor Service**: Processes queued files asynchronously.
- **Redis**: Tracks file processing states.
- **RabbitMQ**: Message queue for job dispatch.

---

## Setup

### Prerequisites

#### For Docker Deployment:
- Docker & Docker Compose

#### For Local Development:
- **API Service**:
  - Node.js & npm
  - Redis
  - RabbitMQ
- **Extractor Service**:
  - Rust & Cargo
  - Redis
  - RabbitMQ
  - Tesseract OCR

---

### Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/codad5/pdfz.git
   cd pdfz
   ```

2. Create an `.env` file for environment variables:
   ```sh
   cp .env.example .env
   ```

3. Update `.env` variables (e.g., ports, RabbitMQ, Redis credentials).

4. Build and start the services:
   ```sh
   docker-compose up --build
   ```

---

## Services & Environment Variables

### Extractor Service (Rust)

- `RUST_LOG=debug` - Log level
- `REDIS_URL` - Redis connection URL
- `RABBITMQ_URL` - RabbitMQ connection
- `EXTRACTOR_PORT` - Service port
- `SHARED_STORAGE_PATH` - Mounted storage
- `TRAINING_DATA_PATH` - Path to Tesseract training data

### API Service (Node.js)

- `NODE_ENV=development`
- `REDIS_URL` - Redis connection
- `RABBITMQ_URL` - RabbitMQ connection
- `API_PORT` - API listening port
- `SHARED_STORAGE_PATH` - Mounted storage

---

## API Endpoints

### Upload a File

```http
POST /upload
```

**Request:** Multipart form-data with a `pdf` file.

**Response:**
```json
{
  "success": true,
  "message": "File uploaded successfully",
  "data": {
    "id": "file-id",
    "filename": "file.pdf",
    "path": "/shared_storage/upload/pdf/file.pdf",
    "size": 12345
  }
}
```

---

### Process a File

```http
POST /process/:id
```

**Request:** JSON body
```json
{
  "startPage": 1,
  "pageCount": 10,
  "priority": 1
}
```

**Response:**
```json
{
  "success": true,
  "message": "File processing started",
  "data": {
    "id": "file-id",
    "file": "file.pdf",
    "options": {
      "startPage": 1,
      "pageCount": 10,
      "priority": 1
    },
    "status": "queued",
    "progress": 0,
    "queuedAt": "2023-10-01T12:00:00Z"
  }
}
```

---

### Track Progress

```http
GET /progress/:id
```

**Response:**
```json
{
  "success": true,
  "message": "Progress retrieved successfully",
  "data": {
    "id": "file-id",
    "progress": 50,
    "status": "processing"
  }
}
```

---

### Retrieve Processed Content

```http
GET /content/:id
```

**Response:**
```json
{
  "success": true,
  "message": "Processed content retrieved successfully",
  "data": {
    "id": "file-id",
    "content": [
      {
        "page_num": 1,
        "text": "This is the text from page 1."
      },
      {
        "page_num": 2,
        "text": "This is the text from page 2."
      }
    ],
    "status": "completed"
  }
}
```

---

## Local Development

### Running the API Locally

1. Install dependencies:
   ```sh
   cd api
   npm install
   ```

2. Start the API service:
   ```sh
   npm run dev
   ```

3. Ensure Redis and RabbitMQ are running locally.

---

### Running the Extractor Locally

1. Install dependencies:
   ```sh
   cd extractor
   cargo build
   ```

2. Install Tesseract OCR:
   - On Ubuntu:
     ```sh
     sudo apt install tesseract-ocr
     ```
   - On macOS:
     ```sh
     brew install tesseract
     ```

3. Start the Extractor service:
   ```sh
   cargo run
   ```

4. Ensure Redis and RabbitMQ are running locally.

---

## Docker Compose Setup

The `docker-compose.yml` file defines the following services:

- **extractor**: Rust-based service for processing PDFs.
- **api**: Node.js-based service for handling API requests.
- **redis**: Redis instance for caching and tracking progress.
- **rabbitmq**: RabbitMQ instance for message queuing.

### Volumes:
- `cargo-cache`: Caches Rust dependencies.
- `training_data`: Stores Tesseract training data.
- `redis-data`: Persists Redis data.
- `shared_storage`: Shared storage for uploaded and processed files.
- `rabbitmq-data`: Persists RabbitMQ data.

---

## Repository

For more details, visit the [GitHub repository](https://github.com/codad5/pdfz).

---

## Contributing

1. Fork the repository and create a new branch.
2. Make changes and test locally.
3. Submit a pull request.

---

## License

MIT License