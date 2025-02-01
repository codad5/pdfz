# PDFz

Developed by [codad5](https://github.com/codad5)

This project consists of a microservices-based system for extracting and processing PDF files. It includes:

- **Extractor Service** (Rust): Handles file processing using Tesseract OCR.
- **API Service** (Node.js): Provides endpoints for uploading and managing file extraction.
- **Redis**: Caching and tracking progress.
- **RabbitMQ**: Message queuing between API and Extractor.
- **Docker**: Containerized deployment for all services.

## Features

- Upload PDF files via the API
- Queue files for processing
- Extract text from PDFs using OCR (Tesseract)
- Track file processing progress
- Store extracted data

## Architecture

- **API Service**: Handles file uploads and processing requests.
- **Extractor Service**: Processes queued files asynchronously.
- **Redis**: Tracks file processing states.
- **RabbitMQ**: Message queue for job dispatch.

## Setup

### Prerequisites

- Docker & Docker Compose
- Node.js & npm (for local API development)
- Rust & Cargo (for local Extractor development)

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

## Services & Environment Variables

### Extractor Service (Rust)

- `RUST_LOG=debug` - Log level
- `REDIS_URL` - Redis connection URL
- `RABBITMQ_URL` - RabbitMQ connection
- `EXTRACTOR_PORT` - Service port
- `SHARED_STORAGE_PATH` - Mounted storage

### API Service (Node.js)

- `NODE_ENV=development`
- `REDIS_URL` - Redis connection
- `RABBITMQ_URL` - RabbitMQ connection
- `API_PORT` - API listening port
- `EXTRACTOR_URL` - URL of the Extractor service

## API Endpoints

### Upload a File

```http
POST /upload
```

**Request:** Multipart form-data with a `pdf` file.



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

## Local Development

### Running the API Locally

```sh
cd api
npm install
npm run dev
```

### Running the Extractor Locally

```sh
cd extractor
cargo run
```

## Repository

For more details, visit the [GitHub repository](https://github.com/codad5/pdfz).

## Contributing

1. Fork the repository and create a new branch.
2. Make changes and test locally.
3. Submit a pull request.

## License

MIT License