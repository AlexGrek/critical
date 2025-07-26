use crit_resource_macro::custom_resource;

custom_resource! {
    pub struct X {
        a: String,
        b: String,
    }
}
