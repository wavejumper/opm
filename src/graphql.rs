use juniper::{FieldResult};

#[derive(juniper::GraphQLObject)]
#[graphql(description="A kit containing up to 24 samples")]
pub struct Sample {
    pub name: String
}

#[derive(juniper::GraphQLObject)]
#[graphql(description="A kit containing up to 24 samples")]
pub struct Kit {
    pub name: String,
    pub dir_name: String,
    pub samples: Vec<Sample>
}

#[derive(juniper::GraphQLInputObject)]
#[graphql(description = "A humanoid creature in the Star Wars universe")]
pub struct NewKit {
    pub name: String
}

#[derive(Clone)]
pub struct Context {
    pub sample_dir: String
}

impl juniper::Context for Context {}

pub struct Query;

graphql_object!(Query: Context |&self| {

    field apiVersion() -> &str {
        "1.0"
    }

    // Arguments to resolvers can either be simple types or input objects.
    // The executor is a special (optional) argument that allows accessing the context.
    field kit(&executor, id: String) -> FieldResult<Kit> {
        let samples: Vec<Sample> = Vec::new();
        let dir_name = id.clone();
        let kit = Kit { name: id, dir_name: dir_name, samples: samples };
        Ok(kit)
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