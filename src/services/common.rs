use crate::prisma::PrismaClient;

pub async fn create_prisma_client() -> Result<PrismaClient, String> {
    match PrismaClient::_builder().build().await {
        Ok(client) => Ok(client),
        Err(err) => {
            log::error!(target: "PrismaClient", "Failed to create Prisma client: {:?}", err);
            Err(err.to_string())
        }
    }
}