#[macro_use]
extern crate downcast;

use downcast::{Any, Downcasted, Downcasted2};
use std::cell::{self, RefCell};
use std::sync::Arc;

/* Trait */

pub trait Animal: Any {}

downcast!(Animal);

/* Impl */

pub struct Bird;

impl Animal for Bird {}

impl Bird {
    fn wash_beak(&self) {
        println!("Beak has been washed! What a clean beak!");
    }
}

/* Main */

fn main() {
    let animal: Arc<Animal> = Arc::new(Bird);
    let bird: Downcasted<Bird, Arc<Animal>> = animal.into();
    bird.wash_beak();

    let animal: Arc<Box<Animal>> = Arc::new(Box::new(Bird));
    let bird: Downcasted2<Bird, Arc<Box<Animal>>> = animal.into();
    bird.wash_beak();

    let animal: RefCell<Box<Animal>> = RefCell::new(Box::new(Bird));
    let bird: Downcasted2<Bird, cell::Ref<Box<Animal>>> = animal.borrow().into();
    bird.wash_beak();
}
