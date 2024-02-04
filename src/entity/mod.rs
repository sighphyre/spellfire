pub mod character;
pub mod location;
pub mod encounter;

trait Describe<T> {
    fn describe(&self, input: T) -> String;
}

pub trait SelfDescribe {
    type Input;

    fn describe(&self, input: &Self::Input) -> String;
}
