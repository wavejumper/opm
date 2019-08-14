use juniper::{FieldResult};
use super::types::{Manifest, Kit, Sample, NewKit};
use crossbeam_utils::atomic::AtomicCell;
use std::sync::Arc;
use crossbeam::channel::Sender;

#[derive(Clone)]
pub struct Context {
    pub sample_dir: String,
    pub manifest: Arc<AtomicCell<Manifest>>,
    pub db_sender: Sender<Manifest>
}

impl juniper::Context for Context {}

pub struct Query;

fn resolve_kit(id: &String) -> Option<Kit> {
    let samples: Vec<Sample> = Vec::new();
    let kit_name = id.clone();
    let dir_name = id.clone();
    let kit = Kit { name: kit_name, dir_name: dir_name, samples: samples };
    Some(kit)
}

graphql_object!(Query: Context |&self| {

    field apiVersion() -> &str {
        "1.0"
    }

    // Arguments to resolvers can either be simple types or input objects.
    // The executor is a special (optional) argument that allows accessing the context.
    field kit(&executor, id: String) -> FieldResult<Option<Kit>> {
        let ctx = executor.context();
        let manifest = ctx.manifest.clone();

        Ok(resolve_kit(&id))
    }
});

pub struct Mutation;

graphql_object!(Mutation: Context |&self| {

    field createKit(&executor, new_kit: NewKit) -> FieldResult<Kit> {
        let samples: Vec<Sample> = Vec::new();
        let dir_name = new_kit.name.clone();
        let kit = Kit { name: new_kit.name, dir_name: dir_name, samples: samples };
        Ok(kit)
    }
});


// A root schema consists of a query and a mutation.
// Request queries can be executed against a RootNode.
pub type Schema = juniper::RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}