pub mod queue;

use tonic::{transport::Server, Request, Response, Status};
use extractor::extractor_service_server::{ExtractorService, ExtractorServiceServer};
use extractor::{ExtractionResponse, ExtractRequest};

pub mod extractor {
    tonic::include_proto!("extractor");
}

#[derive(Debug, Default)]
pub struct MyExtractor {}

#[tonic::async_trait]
impl ExtractorService for MyExtractor {
    async fn extract_text(
        &self,
        request: Request<ExtractRequest>,
    ) -> Result<Response<ExtractionResponse>, Status> {
        let name = request.into_inner().file_path;
        println!("Got a request: {:?}", name);
        let reply = extractor::ExtractionResponse {
            job_id: format!("extractor, {}!", name),
            status: "200".to_string(),
            text: "insert text here".to_string(),
            // progress: 0.0,
        };
        Ok(Response::new(reply))
    }

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let addr = "[::1]:50051".parse().unwrap();
    let addr = "0.0.0.0:50051".parse().unwrap(); // Bind to all interfaces
    let extractor = MyExtractor::default();

    println!("ExtractorServiceServer listening on {}", addr);

    Server::builder()
        .add_service(ExtractorServiceServer::new(extractor))
        .serve(addr)
        .await?;

    Ok(())
}
