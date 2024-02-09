pub mod character;
pub mod encounter;
pub mod location;

trait Describe<T> {
    fn describe(&self, input: T) -> String;
}

pub trait SelfDescribe {
    type Input;

    fn describe(&self, input: &Self::Input) -> String;
}
