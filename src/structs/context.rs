use std::sync::Arc;

use twilight_http::{client::InteractionClient, Client};
use twilight_model::id::{marker::ApplicationMarker, Id};

use super::{cache::Cache, database::Database};

pub struct Context {
    pub application_id: Id<ApplicationMarker>,
    pub cache: Cache,
    pub client: Arc<Client>,
    pub database: Database,
}

impl Context {
    pub fn interaction_client(&self) -> InteractionClient {
        self.client.interaction(self.application_id)
    }

    pub fn new(application_id: Id<ApplicationMarker>, client: Client) -> Self {
        Self {
            application_id,
            cache: Cache::new(),
            client: Arc::new(client),
            database: Database::new(),
        }
    }
}
