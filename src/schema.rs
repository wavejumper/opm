
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