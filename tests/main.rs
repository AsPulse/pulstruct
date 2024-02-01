use pulstruct::pulstruct_api;

struct Test {
  a: String,
}

#[pulstruct_api(TestApi)]
impl Test {
  pub fn some_procedure(&self, a: String, b: String) -> bool {
    a == b
  }
}
